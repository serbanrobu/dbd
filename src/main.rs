use async_std::prelude::*;
use async_std::{eprintln, io, process, task};
use console::{style, Emoji};
use indicatif::{HumanBytes, HumanDuration, ProgressBar, ProgressStyle};
use std::time::Instant;
use structopt::StructOpt;
use surf::{Error, Result, StatusCode, Url};
use uuid::Uuid;

/// Client for fetching database dumps
#[derive(StructOpt, Debug)]
struct Opt {
    /// Database ID configured from the agent
    #[structopt(env)]
    database_id: String,
    /// Agent URL
    #[structopt(short, long, env)]
    url: Url,
    /// Key for accessing agent's API
    #[structopt(short = "k", long, env, hide_env_values = true)]
    api_key: String,
    /// Do not dump the specified table data. To specify more than one table to
    /// ignore, use comma separator, e.g. --exclude-table-data=table_1,table_2.
    #[structopt(long, env)]
    exclude_table_data: Option<String>,
}

static PAPER: Emoji<'_, '_> = Emoji("📃 ", "");
static SPARKLE: Emoji<'_, '_> = Emoji("✨ ", ":-)");

#[async_std::main]
async fn main() -> Result<()> {
    let opts = Opt::from_args();
    let started = Instant::now();
    let mut url = opts
        .url
        .join(&format!("databases/{}/dump", opts.database_id))?;

    if let Some(ref tables) = opts.exclude_table_data {
        url.query_pairs_mut()
            .append_pair("exclude_table_data", tables);
    }

    let mut res = surf::get(url).header("x-api-key", &opts.api_key).await?;
    let body = res.body_string().await?;
    let cmd_id = match res.status() {
        StatusCode::Ok => body.parse::<Uuid>()?,
        status => return Err(Error::from_str(status, body)),
    };

    let base_url = opts.url.join(&format!("commands/{}/", cmd_id))?;

    let url = base_url.join("stdout")?;
    let req = surf::get(url).header("x-api-key", &opts.api_key);
    let t1 = task::spawn(async move {
        let mut res = req.await?;
        if res.status() != StatusCode::Ok {
            let msg = res.body_string().await?;
            return Err(Error::from_str(res.status(), msg));
        }

        let total = io::copy(res, io::stdout()).await?;
        Ok(total)
    });

    console::set_colors_enabled(true);

    let url = base_url.join("stderr")?;
    let req = surf::get(url).header("x-api-key", &opts.api_key);
    let t2 = task::spawn(async move {
        let mut res = req.await?;
        if res.status() != StatusCode::Ok {
            let msg = res.body_string().await?;
            return Err(Error::from_str(res.status(), msg));
        }

        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(120);
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{prefix}{msg} {spinner:.blue} {elapsed:.yellow}"),
        );
        pb.set_prefix(&PAPER.to_string());
        pb.set_message("Dumping...");

        let mut lines = res.lines();
        while let Some(line) = lines.next().await.transpose()? {
            pb.println(line);
        }

        pb.finish_and_clear();

        Ok(())
    });

    let mut res = surf::get(base_url.join("status")?)
        .header("x-api-key", &opts.api_key)
        .await?;

    let (total, _) = t1.try_join(t2).await?;

    let body = res.body_string().await?;
    let code = match res.status() {
        StatusCode::Ok => body.parse::<i32>()?,
        status => return Err(Error::from_str(status, body)),
    };

    if code == 0 {
        eprintln!(
            "{}Done in {} ({})",
            SPARKLE,
            HumanDuration(started.elapsed()),
            style(HumanBytes(total as u64)).cyan(),
        )
        .await;
    }

    process::exit(code);
}
