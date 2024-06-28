pub mod cmd;
pub use cmd::Command;

mod connection;
pub use connection::Connection;

mod db;
use db::Db;

pub mod frame;
pub use frame::Frame;

mod parse;
use parse::Parse;

pub mod server;

mod shutdown;
use shutdown::Shutdown;

/// Default port that a redis server listens on.
pub const DEFAULT_PORT: u16 = 6379;

/// Default buffer size for a connection.
pub const PROTO_MAX_BULK_LEN: usize = 512;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
