use crate::settings::Database;
use anyhow::{Context, Result};
use async_std::io;
use async_std::process::{Child, Command, Stdio};
use async_std::task;

pub fn mysqldump(db: &Database) -> Result<Child> {
    let mut cmd = Command::new("mysqldump");
    cmd.args(&[
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
    ]);

    if let Some(ref tables) = db.exclude_table_data {
        for table in tables {
            cmd.args(&["--ignore-table-data", &format!("{}.{}", db.dbname, table)]);
        }
    }

    let mut mysqldump = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to run mysqldump")?;

    let mut gzip = Command::new("gzip")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to run gzip")?;

    gzip.stderr = mysqldump.stderr;

    task::spawn(io::copy(
        mysqldump.stdout.take().unwrap(),
        gzip.stdin.take().unwrap(),
    ));

    Ok(gzip)
}

pub fn pg_dump(db: &Database) -> Result<Child> {
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

    if let Some(ref tables) = db.exclude_table_data {
        for table in tables {
            cmd.args(&["--exclude-table-data", &table]);
        }
    }

    cmd.env("PGPASSWORD", &db.password)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to run pg_dump")
}
