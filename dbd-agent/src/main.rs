use anyhow::Context;
use async_std::io::BufReader;
use dbd_agent::{configure, encode, mysqldump, pg_dump};
use dbd_agent::{AuthMiddleware, ConnectionKind, DumpQuery, Opt, Settings};
use structopt::StructOpt;
use tide::utils::After;
use tide::{Body, Error, Request, Response, Result, StatusCode};
use tokio_util::compat::TokioAsyncReadCompatExt;

async fn dump(req: Request<Settings>) -> Result<Body> {
    let conn_id = req.param("connection_id")?;
    let conn = req
        .state()
        .connections
        .get(conn_id)
        .with_context(|| format!("no database connection {}", conn_id))
        .map_err(|e| Error::new(StatusCode::NotFound, e))?;

    let dbname = req
        .param("dbname")
        .ok()
        .or(conn.dbname.as_ref().map(String::as_str))
        .context("no database name provided")
        .map_err(|e| Error::new(StatusCode::NotFound, e))?;

    let DumpQuery { exclude_table_data } = req.query()?;
    let read = match conn.kind {
        ConnectionKind::Postgres => encode(pg_dump(conn, dbname, exclude_table_data)?),
        ConnectionKind::MySql => encode(mysqldump(conn, dbname, exclude_table_data)?),
    };

    Ok(Body::from_reader(BufReader::new(read.compat()), None))
}

#[async_std::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();
    let settings = configure(opt.config)?;
    let addr = format!("{}:{}", settings.address, settings.port);

    tide::log::start();
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
