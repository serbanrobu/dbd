use anyhow::{bail, Context, Error, Result};
use async_bincode::AsyncBincodeReader;
use async_std::prelude::*;
use async_std::sync::Arc;
use async_std::task;
use console::style;
use dbd::emoji::{PAPER, SPARKLE};
use dbd::{configure, Agent, Opt};
use dbd_agent::Frame;
use indicatif::{HumanBytes, HumanDuration, ProgressBar, ProgressStyle};
use std::time::Instant;
use std::{io, process};
use structopt::StructOpt;
use surf::StatusCode;
use tokio::io::{duplex, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio_util::compat::FuturesAsyncReadCompatExt;

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
            .map(|s| s.agents.get(&opt.connection_id))
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

    let mut url = agent.url.join(&format!(
        "dump/{}/{}",
        opt.connection_id,
        opt.dbname.as_ref().map(String::as_str).unwrap_or("")
    ))?;

    if let Some(ref tables) = opt.exclude_table_data {
        url.query_pairs_mut()
            .append_pair("exclude_table_data", tables);
    }

    let mut res = surf::get(url)
        .header("x-api-key", &agent.api_key)
        .await
        .map_err(Error::msg)?;

    if res.status() != StatusCode::Ok {
        let msg = res.body_string().await.map_err(Error::msg)?;
        bail!(msg);
    }

    let mut stream = AsyncBincodeReader::from(res.compat());
    let mut total = 0;

    console::set_colors_enabled(true);

    let pb = Arc::new(ProgressBar::new_spinner());
    pb.enable_steady_tick(120);
    pb.set_prefix(&PAPER.to_string());
    pb.set_message("Dumping...");
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{prefix}{msg} {spinner:.magenta} {bytes:.cyan} ({bytes_per_sec:.dim}) [{elapsed_precise:.yellow}]"),
    );

    let (reader, mut writer) = duplex(1024);
    let pb2 = pb.clone();
    let handle = task::spawn(async move {
        let mut lines = BufReader::new(reader).lines();

        while let Some(line) = lines.next_line().await? {
            pb2.println(line);
        }

        Ok::<_, Error>(())
    });

    while let Some(frame) = stream.next().await {
        match frame? {
            Frame::Stdout(bytes) => {
                total += io::copy(&mut pb.wrap_read(&bytes[..]), &mut io::stdout())?;
            }
            Frame::Stderr(bytes) => {
                writer.write_all(&bytes).await?;
            }
            Frame::Status(code) => {
                writer.shutdown().await?;
                handle.await?;
                pb.finish_and_clear();

                match code {
                    Some(0) => {
                        eprintln!(
                            "{}Done in {} ({})",
                            SPARKLE,
                            HumanDuration(started.elapsed()),
                            style(HumanBytes(total as u64)).cyan(),
                        );

                        break;
                    }
                    Some(c) => process::exit(c),
                    _ => bail!("process interrupted"),
                }
            }
        }
    }

    Ok(())
}
