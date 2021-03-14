use crate::{Connection, Frame};
use anyhow::{Context, Result};
use async_compression::tokio::bufread::GzipEncoder;
use async_compression::Level;
use futures::FutureExt;
use std::process::Stdio;
use tokio::io::{self, AsyncRead, AsyncReadExt, BufReader};
use tokio::process::Command;
use tokio_stream::{Stream, StreamExt};
use tokio_util::codec::{BytesCodec, FramedRead};

fn into_frame_stream(
    reader: impl AsyncRead,
    op: fn(Vec<u8>) -> Frame,
) -> impl Stream<Item = io::Result<Frame>> {
    FramedRead::new(reader, BytesCodec::new()).map(move |r| r.map(|b| b.to_vec()).map(op))
}

pub fn mysqldump(
    conn: &Connection,
    dbname: &str,
    exclude_table_data: Option<Vec<String>>,
) -> Result<impl Stream<Item = io::Result<Frame>>> {
    let args = [
        "-h",
        &conn.host,
        "-P",
        &conn.port.to_string(),
        "-u",
        &conn.username,
        &format!("-p{}", conn.password),
        "-v",
        "--single-transaction",
        dbname,
    ];

    let mut dump_schema = Command::new("mysqldump")
        .args(&args)
        .arg("--no-data")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to spawn mysqldump")?;

    let mut cmd = Command::new("mysqldump");

    if let Some(tables) = exclude_table_data {
        for table in tables {
            cmd.args(&["--ignore-table", &format!("{}.{}", dbname, table)]);
        }
    }

    let mut dump_data = cmd
        .args(&args)
        .arg("--no-create-info")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to run mysqldump")?;

    let stdout = into_frame_stream(
        GzipEncoder::with_quality(
            BufReader::new(
                dump_schema
                    .stdout
                    .take()
                    .unwrap()
                    .chain(dump_data.stdout.take().unwrap()),
            ),
            Level::Fastest,
        ),
        Frame::Stdout,
    );

    let stderr = into_frame_stream(
        dump_schema
            .stderr
            .take()
            .unwrap()
            .chain(dump_data.stderr.take().unwrap()),
        Frame::Stderr,
    );

    let status = Box::pin(async move {
        let status = dump_schema.wait().await?;

        if status.code().map_or(true, |c| c != 0) {
            return Ok(Frame::Status(status.code()));
        }

        let status = dump_data.wait().await?;
        Ok(Frame::Status(status.code()))
    });

    Ok(stdout.merge(stderr).chain(status.into_stream()))
}

pub fn pg_dump(
    conn: &Connection,
    dbname: &str,
    exclude_table_data: Option<Vec<String>>,
) -> Result<impl Stream<Item = io::Result<Frame>>> {
    let mut cmd = Command::new("pg_dump");
    cmd.args(&[
        "-d",
        dbname,
        "-h",
        &conn.host,
        "-p",
        &conn.port.to_string(),
        "-U",
        &conn.username,
        "-v",
        "-Z",
        "1",
    ]);

    if let Some(tables) = exclude_table_data {
        for table in tables {
            cmd.args(&["--exclude-table-data", &table]);
        }
    }

    let mut child = cmd
        .env("PGPASSWORD", &conn.password)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to spawn pg_dump")?;

    let stdout = into_frame_stream(child.stdout.take().unwrap(), Frame::Stdout);
    let stderr = into_frame_stream(child.stderr.take().unwrap(), Frame::Stderr);

    let status = Box::pin(async move {
        let status = child.wait().await?;
        Ok(Frame::Status(status.code()))
    });

    Ok(stdout.merge(stderr).chain(status.into_stream()))
}
