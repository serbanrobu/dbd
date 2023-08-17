mod auth;
mod commands;
mod dump_query;
mod frame;
mod opt;
mod settings;
mod util;

pub use auth::AuthMiddleware;
pub use commands::{mysqldump_data, mysqldump_schema, pg_dump};
pub use dump_query::DumpQuery;
pub use frame::Frame;
pub use opt::Opt;
pub use settings::{configure, Connection, ConnectionKind, Settings};
pub use util::command_exists;

pub const BUF_SIZE: usize = 1 + 8 + 64 * 1024;
