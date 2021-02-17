use anyhow::Context;
use async_std::io::{self, BufReader};
use async_std::sync::Arc;
use async_std::task;
use dbd::auth::AuthMiddleware;
use dbd::commands::{mysqldump, pg_dump};
use dbd::settings::configure;
use dbd::{Connection, State};
use std::path::PathBuf;
use std::time::Duration;
use structopt::StructOpt;
use tide::utils::After;
use tide::{Body, Error, Request, Response, Result, StatusCode};
use uuid::Uuid;

/// Agent for serving database dumps
#[derive(StructOpt)]
struct Opt {
    /// Config file name. Defaults to $HOME/.dbd-agent.*
    #[structopt(short, long, env, parse(from_os_str))]
    config: Option<PathBuf>,
}

#[async_std::main]
async fn main() -> Result<()> {
    let opts = Opt::from_args();
    let settings = configure(opts.config)?;
    let addr = format!("{}:{}", settings.address, settings.port);
    let state = Arc::new(State::new(settings));

    tide::log::start();
    let mut app = tide::with_state(state.clone());

    app.with(AuthMiddleware);

    app.at("/databases/:id/dump")
        .get(|req: Request<Arc<State>>| async move {
            let db_id = req.param("id")?;
            let db = req.state().settings.databases.get(db_id).ok_or_else(|| {
                Error::from_str(StatusCode::NotFound, format!("no database {}", db_id))
            })?;

            let (child, stdout, stderr) = match db.connection {
                Connection::Postgres => pg_dump(db)?,
                Connection::MySql => mysqldump(db)?,
            };

            let state = req.state();
            let cmd_id = Uuid::new_v4();

            let mut stdouts = state.stdouts.lock().await;
            stdouts.insert(cmd_id, stdout);
            drop(stdouts);

            let mut stderrs = state.stderrs.lock().await;
            stderrs.insert(cmd_id, stderr);
            drop(stderrs);

            let mut cmds = state.commands.lock().await;
            cmds.insert(cmd_id, child);
            drop(cmds);

            task::spawn(async move {
                task::sleep(Duration::from_secs(5)).await;

                let state = req.state();

                let mut stdouts = state.stdouts.lock().await;
                stdouts.remove(&cmd_id);
                drop(stdouts);

                let mut stderrs = state.stderrs.lock().await;
                stderrs.remove(&cmd_id);
                drop(stderrs);

                let mut cmds = state.commands.lock().await;
                cmds.remove(&cmd_id);
                drop(cmds);
            });

            Ok(cmd_id.to_string())
        });

    app.at("/commands/:id/status")
        .get(|req: Request<Arc<State>>| async move {
            let id = req.param("id")?.parse::<Uuid>()?;

            let mut cmds = req.state().commands.lock().await;
            let mut cmd = cmds
                .remove(&id)
                .with_context(|| format!("no dump command {}", id))?;

            drop(cmds);

            let status = cmd.status().await?;
            let code = status
                .code()
                .with_context(|| format!("dump command {} was interrupted", id))?;

            Ok(code.to_string())
        });

    app.at("/commands/:id/stdout")
        .get(|req: Request<Arc<State>>| async move {
            let id = req.param("id")?.parse::<Uuid>()?;

            let mut stdouts = req.state().stdouts.lock().await;
            let stdout = stdouts
                .remove(&id)
                .with_context(|| format!("no dump stdout {}", id))?;

            drop(stdouts);

            let reader = BufReader::new(stdout);
            let body = Body::from_reader(reader, None);
            Ok(body)
        });

    app.at("/commands/:id/stderr")
        .get(|req: Request<Arc<State>>| async move {
            let id = req.param("id")?.parse::<Uuid>()?;

            let mut stderrs = req.state().stderrs.lock().await;
            let stderr = stderrs
                .remove(&id)
                .with_context(|| format!("no dump stderr {}", id))?;

            drop(stderrs);

            let reader = BufReader::new(stderr);
            let body = Body::from_reader(reader, None);
            Ok(body)
        });

    app.with(After(|mut res: Response| async {
        if let Some(err) = res.downcast_error::<io::Error>() {
            let msg = err.to_string();
            res.set_body(msg);
        } else if let Some(err) = res.downcast_error::<String>() {
            let msg = err.to_owned();
            res.set_body(msg);
        }

        Ok(res)
    }));

    app.listen(addr).await?;

    Ok(())
}
