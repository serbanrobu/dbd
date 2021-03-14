use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Frame {
    Stdout(Vec<u8>),
    Stderr(Vec<u8>),
    Status(Option<i32>),
}
