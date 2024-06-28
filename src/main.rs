use redis::{server, DEFAULT_PORT};
use std::io;
use tokio::net::TcpListener;
use tokio::signal;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind(&format!("127.0.0.1:{DEFAULT_PORT}")).await?;

    server::run(listener, signal::ctrl_c()).await;

    Ok(())
}
