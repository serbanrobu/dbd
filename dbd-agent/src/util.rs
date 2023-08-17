use std::{env, fs};

pub fn command_exists(name: &str) -> bool {
    if let Ok(path) = env::var("PATH") {
        for p in path.split(':') {
            if fs::metadata(format!("{}/{}", p, name)).is_ok() {
                return true;
            }
        }
    }

    false
}
