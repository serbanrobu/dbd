use std::path::PathBuf;
use structopt::StructOpt;

/// Agent for serving database dumps
#[derive(StructOpt)]
pub struct Opt {
    /// Config file name. Defaults to $HOME/.dbd-agent.toml
    #[structopt(short, long, env, parse(from_os_str))]
    pub config: Option<PathBuf>,
}
