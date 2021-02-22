use crate::Database;
use anyhow::{Context, Result};
use async_std::io;
use async_std::io::prelude::*;
use async_std::process::{Child, ChildStdout, Command, Stdio};
use async_std::task;

pub type DumpStderr = Box<dyn Read + Send + Sync + Unpin>;

pub fn mysqldump(
    db: &Database,
    exclude_table_data: Option<Vec<String>>,
) -> Result<(Child, ChildStdout, DumpStderr)> {
    let args = [
        "-h",
        &db.host,
        "-P",
        &db.port.to_string(),
        "-u",
        &db.username,
        &format!("-p{}", db.password),
        "-v",
        &db.dbname,
        "--single-transaction",
    ];

    let dump_schema = Command::new("mysqldump")
        .args(&args)
        .arg("--no-data")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to run mysqldump")?;

    let mut cmd = Command::new("mysqldump");

    if let Some(tables) = exclude_table_data {
        for table in tables {
            cmd.args(&["--ignore-table", &format!("{}.{}", db.dbname, table)]);
        }
    }

    let dump_data = cmd
        .args(&args)
        .arg("--no-create-info")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to run mysqldump")?;

    let mut gzip = Command::new("gzip")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("failed to run gzip")?;

    task::spawn(io::copy(
        dump_schema.stdout.unwrap().chain(dump_data.stdout.unwrap()),
        gzip.stdin.take().unwrap(),
    ));

    let gzip_stdout = gzip.stdout.take().unwrap();
    let stderr = dump_schema.stderr.unwrap().chain(dump_data.stderr.unwrap());

    Ok((gzip, gzip_stdout, Box::new(stderr)))
}

pub fn pg_dump(
    db: &Database,
    exclude_table_data: Option<Vec<String>>,
) -> Result<(Child, ChildStdout, DumpStderr)> {
    let mut cmd = Command::new("pg_dump");
    cmd.args(&[
        "-d",
        &db.dbname,
        "-h",
        &db.host,
        "-p",
        &db.port.to_string(),
        "-U",
        &db.username,
        "-v",
        "-Z",
        "9",
    ]);

    if let Some(tables) = exclude_table_data {
        for table in tables {
            cmd.args(&["--exclude-table-data", &table]);
        }
    }

    let mut child = cmd
        .env("PGPASSWORD", &db.password)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to run pg_dump")?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    Ok((child, stdout, Box::new(stderr)))
}
