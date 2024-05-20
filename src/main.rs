use redis_starter_rust;
use std::io;
use tokio::net::TcpListener;
use tokio::signal;

// TODO: move this
const DEFAULT_PORT: u16 = 6379;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind(&format!("127.0.0.1:{}", DEFAULT_PORT)).await?;

    redis_starter_rust::run(listener, signal::ctrl_c()).await;

    Ok(())
}
