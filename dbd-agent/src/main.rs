use anyhow::Context;
use async_bincode::AsyncBincodeWriter;
use async_std::io::BufReader;
use async_std::task;
use chrono::Local;
use dbd_agent::{
    command_exists, configure, mysqldump_data, mysqldump_schema, pg_dump, Frame, BUF_SIZE,
};
use dbd_agent::{AuthMiddleware, ConnectionKind, DumpQuery, Opt, Settings};
use futures::{SinkExt, Stream, TryStreamExt};
use structopt::StructOpt;
use tide::{log, utils::After, Body, Error, Request, Response, Result, StatusCode};
use tokio::io::duplex;
use tokio::process::Child;
use tokio_stream::StreamExt;
use tokio_util::compat::TokioAsyncReadCompatExt;
use tokio_util::io::ReaderStream;

async fn dump(req: Request<Settings>) -> Result<Body> {
    let conn_id = req.param("connection_id")?;
    let state = req.state();
    let conn = state
        .connections
        .get(conn_id)
        .with_context(|| format!("no database connection {}", conn_id))
        .map_err(|e| Error::new(StatusCode::NotFound, e))?;

    let dbname = req
        .param("dbname")
        .ok()
        .or(conn.dbname.as_deref())
        .context("no database name provided")
        .map_err(|e| Error::new(StatusCode::NotFound, e))?;

    let DumpQuery { exclude_table_data } = req.query()?;

    let (reader, writer) = duplex(BUF_SIZE);

    let mut sink = AsyncBincodeWriter::from(writer)
        .for_async()
        .sink_map_err(Error::from);

    match conn.kind {
        ConnectionKind::Postgres => {
            let mut pg_dump = pg_dump(conn, dbname, exclude_table_data);

            let future = async move {
                let mut child = pg_dump.spawn().context("failed to spawn pg_dump")?;
                sink.send_all(&mut child_stream(&mut child)).await?;
                let status = child.wait().await?;
                sink.send(Frame::Status(status.code())).await
            };

            task::spawn(async move {
                if let Err(e) = future.await {
                    log::error!("{}", e);
                }
            });
        }
        ConnectionKind::MySql => {
            let mut mysqldump_schema = mysqldump_schema(conn, dbname);
            let mut mysqldump_data = mysqldump_data(conn, dbname, exclude_table_data);

            let future = async move {
                let mut child = mysqldump_schema
                    .spawn()
                    .context("failed to spawn mysqldump")?;

                sink.send_all(&mut child_stream(&mut child)).await?;
                let status = child.wait().await?;

                if status.code().map_or(true, |c| c != 0) {
                    return sink.send(Frame::Status(status.code())).await;
                }

                let mut child = mysqldump_data
                    .spawn()
                    .context("failed to spawn mysqldump")?;

                sink.send_all(&mut child_stream(&mut child)).await?;
                let status = child.wait().await?;
                sink.send(Frame::Status(status.code())).await
            };

            task::spawn(async move {
                if let Err(e) = future.await {
                    log::error!("{}", e);
                }
            });
        }
    };

    log::info!(
        "User {} started to dump the {} database on {}",
        state
            .api_keys
            .get(
                req.header("x-api-key")
                    .context("No api key is provided")?
                    .as_str()
            )
            .context("Invalid api key")?,
        dbname,
        Local::now().format("%b %e, %Y, %H:%M:%S"),
    );

    Ok(Body::from_reader(
        BufReader::with_capacity(BUF_SIZE, reader.compat()),
        None,
    ))
}

fn child_stream(child: &mut Child) -> impl Stream<Item = Result<Frame>> {
    let stdout = ReaderStream::with_capacity(child.stdout.take().unwrap(), BUF_SIZE)
        .map(move |res| res.map(Frame::Stdout));

    let stderr = ReaderStream::with_capacity(child.stderr.take().unwrap(), 1024)
        .map(move |res| res.map(Frame::Stderr));

    stdout.merge(stderr).map_err(Error::from)
}

#[async_std::main]
async fn main() -> Result<()> {
    log::start();

    for cmd in &["pg_dump", "mysqldump"] {
        if !command_exists(cmd) {
            log::warn!("Command `{}` not found in PATH", cmd);
        }
    }

    let opt = Opt::from_args();
    let settings = configure(opt.config)?;
    let addr = format!("{}:{}", settings.address, settings.port);
    let mut app = tide::with_state(settings);

    app.with(AuthMiddleware);
    app.at("/dump/:connection_id/").get(dump);
    app.at("/dump/:connection_id/:dbname").get(dump);
    app.with(After(|mut res: Response| async {
        if let Some(err) = res.take_error() {
            res.set_body(err.to_string());
        }

        Ok(res)
    }));

    app.listen(addr).await?;

    Ok(())
}
