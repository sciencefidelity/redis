use redis_starter_rust::server;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::test]
async fn ping() {
    let addr = start_server().await;

    let mut stream = TcpStream::connect(addr).await.unwrap();

    stream.write_all(b"*1\r\n$4\r\nPING\r\n").await.unwrap();

    let mut response = [0; 7];
    stream.read(&mut response).await.unwrap();
    assert_eq!(b"+PONG\r\n", &response);
}

#[tokio::test]
async fn ping_with_string() {
    let addr = start_server().await;

    let mut stream = TcpStream::connect(addr).await.unwrap();

    stream
        .write_all(b"*2\r\n$4\r\nPING\r\n$3\r\nhey\r\n")
        .await
        .unwrap();

    let mut response = [0; 9];
    stream.read(&mut response).await.unwrap();
    assert_eq!(b"$3\r\nhey\r\n", &response);
}

#[tokio::test]
async fn echo() {
    let addr = start_server().await;

    let mut stream = TcpStream::connect(addr).await.unwrap();

    stream
        .write_all(b"*2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n")
        .await
        .unwrap();

    let mut response = [0; 9];
    stream.read(&mut response).await.unwrap();
    assert_eq!(b"$3\r\nhey\r\n", &response);
}

async fn start_server() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move { server::run(listener).await });

    addr
}
