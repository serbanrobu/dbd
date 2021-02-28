use anyhow::{bail, Context, Error, Result};
use async_std::prelude::*;
use async_std::{eprintln, io, process, task};
use console::style;
use dbd::emoji::{PAPER, SPARKLE};
use dbd::{configure, Agent, Opt};
use indicatif::{HumanBytes, HumanDuration, ProgressBar, ProgressStyle};
use std::time::Instant;
use structopt::StructOpt;
use surf::StatusCode;
use uuid::Uuid;

#[async_std::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();
    let started = Instant::now();
    let settings = configure(opt.config.clone())?;
    let agent = match opt.agent_id.as_ref() {
        Some(id) => Some(
            settings
                .as_ref()
                .context("no configuration provided")?
                .agents
                .get(id)
                .with_context(|| format!("no agent {}", id))?,
        ),
        _ => settings
            .as_ref()
            .map(|s| s.agents.get(&opt.database_id))
            .flatten(),
    };
    let agent = match agent {
        Some(a) => Agent {
            url: opt.url.clone().unwrap_or_else(|| a.url.clone()),
            api_key: opt.api_key.clone().unwrap_or_else(|| a.api_key.clone()),
        },
        _ => Agent {
            url: opt.url.clone().context("no URL provided")?,
            api_key: opt.api_key.clone().context("no api key provided")?,
        },
    };

    let mut url = agent
        .url
        .join(&format!("databases/{}/dump", opt.database_id))?;

    if let Some(ref tables) = opt.exclude_table_data {
        url.query_pairs_mut()
            .append_pair("exclude_table_data", tables);
    }

    let mut res = surf::get(url)
        .header("x-api-key", &agent.api_key)
        .await
        .map_err(Error::msg)?;
    let body = res.body_string().await.map_err(Error::msg)?;
    let cmd_id = match res.status() {
        StatusCode::Ok => body.parse::<Uuid>()?,
        _ => bail!(body),
    };

    let base_url = agent.url.join(&format!("commands/{}/", cmd_id))?;

    let url = base_url.join("stdout")?;
    let req = surf::get(url).header("x-api-key", &agent.api_key);
    let t1 = task::spawn(async move {
        let mut res = req.await.map_err(Error::msg)?;
        if res.status() != StatusCode::Ok {
            let msg = res.body_string().await.map_err(Error::msg)?;
            bail!(msg);
        }

        let total = io::copy(res, io::stdout()).await?;
        Ok(total)
    });

    console::set_colors_enabled(true);

    let url = base_url.join("stderr")?;
    let req = surf::get(url).header("x-api-key", &agent.api_key);
    let t2 = task::spawn(async move {
        let mut res = req.await.map_err(Error::msg)?;
        if res.status() != StatusCode::Ok {
            let msg = res.body_string().await.map_err(Error::msg)?;
            bail!(msg);
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
        .header("x-api-key", &agent.api_key)
        .await
        .map_err(Error::msg)?;

    let (total, _) = t1.try_join(t2).await?;

    let body = res.body_string().await.map_err(Error::msg)?;
    let code = match res.status() {
        StatusCode::Ok => body.parse::<i32>()?,
        _ => bail!(body),
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
