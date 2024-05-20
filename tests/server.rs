use redis_starter_rust as server;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::test]
async fn ping() {
    let addr = start_server().await;

    let mut stream = TcpStream::connect(addr).await.unwrap();

    stream.write_all(b"*2\r\n$3\r\nPING\r\n").await.unwrap();

    let mut response = [0; 7];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(b"+PONG\r\n", &response);
}

async fn start_server() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move { server::run(listener, tokio::signal::ctrl_c()).await });

    addr
}
