use bytes::Bytes;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Frame {
    Stdout(Bytes),
    Stderr(Bytes),
    Status(Option<i32>),
}
