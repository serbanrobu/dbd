use crate::settings::Settings;
use async_std::process::{Child, ChildStderr, ChildStdout};
use async_std::sync::Mutex;
use std::collections::HashMap;
use uuid::Uuid;

pub struct State {
    pub settings: Settings,
    pub commands: Mutex<HashMap<Uuid, Child>>,
    pub stdouts: Mutex<HashMap<Uuid, ChildStdout>>,
    pub stderrs: Mutex<HashMap<Uuid, ChildStderr>>,
}

impl State {
    pub fn new(settings: Settings) -> Self {
        Self {
            settings,
            commands: Mutex::new(HashMap::new()),
            stdouts: Mutex::new(HashMap::new()),
            stderrs: Mutex::new(HashMap::new()),
        }
    }
}
