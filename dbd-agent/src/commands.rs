use tokio::process::Command;

use crate::Connection;
use std::process::Stdio;

pub fn mysqldump_schema(conn: &Connection, dbname: &str) -> Command {
    let mut cmd = Command::new("mysqldump");

    cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).args([
        "-h",
        &conn.host,
        "-P",
        &conn.port.to_string(),
        "-u",
        &conn.username,
        &format!("-p{}", conn.password),
        "-v",
        "--single-transaction",
        "--no-data",
        dbname,
    ]);

    cmd
}

pub fn mysqldump_data(
    conn: &Connection,
    dbname: &str,
    exclude_table_data: Option<Vec<String>>,
) -> Command {
    let mut cmd = Command::new("mysqldump");

    cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).args([
        "-h",
        &conn.host,
        "-P",
        &conn.port.to_string(),
        "-u",
        &conn.username,
        &format!("-p{}", conn.password),
        "-v",
        "--single-transaction",
        "--no-create-info",
        dbname,
    ]);

    if let Some(tables) = exclude_table_data {
        for table in tables {
            cmd.args(["--ignore-table", &format!("{}.{}", dbname, table)]);
        }
    }

    cmd
}

pub fn pg_dump(
    conn: &Connection,
    dbname: &str,
    exclude_table_data: Option<Vec<String>>,
) -> Command {
    let mut cmd = Command::new("pg_dump");

    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("PGPASSWORD", &conn.password)
        .args([
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
            cmd.args(["--exclude-table-data", &table]);
        }
    }

    cmd
}
