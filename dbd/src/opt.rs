use std::path::PathBuf;
use structopt::StructOpt;
use surf::Url;

/// Client for fetching database dumps
#[derive(StructOpt, Debug)]
pub struct Opt {
    /// Agent ID from configuration. Defaults to database ID if the ids match.
    #[structopt(short, long, env)]
    pub agent_id: Option<String>,
    /// Key for accessing agent's API
    #[structopt(short = "k", long, env, hide_env_values = true)]
    pub api_key: Option<String>,
    /// Config file name. Defaults to $HOME/.dbd.toml
    #[structopt(short, long, env, parse(from_os_str))]
    pub config: Option<PathBuf>,
    /// Do not dump the specified table data. To specify more than one table to
    /// ignore, use comma separator, e.g. --exclude-table-data=table_1,table_2.
    #[structopt(short, long, env)]
    pub exclude_table_data: Option<String>,
    /// Agent URL
    #[structopt(short, long, env)]
    pub url: Option<Url>,
    /// Database ID configured from the agent
    #[structopt(env)]
    pub database_id: String,
}
