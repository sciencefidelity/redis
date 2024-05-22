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
async fn ping_concurrently() {
    let addr1 = start_server().await;
    let addr2 = start_server().await;

    let mut stream1 = TcpStream::connect(addr1).await.unwrap();
    let mut stream2 = TcpStream::connect(addr2).await.unwrap();

    stream1.write_all(b"*1\r\n$4\r\nPING\r\n").await.unwrap();
    stream2.write_all(b"*1\r\n$4\r\nPING\r\n").await.unwrap();

    let mut response1 = [0; 7];
    let mut response2 = [0; 7];
    stream1.read(&mut response1).await.unwrap();
    stream2.read(&mut response2).await.unwrap();
    stream1.write_all(b"*1\r\n$4\r\nPING\r\n").await.unwrap();
    let mut response3 = [0; 7];
    stream1.read(&mut response3).await.unwrap();
    assert_eq!(b"+PONG\r\n", &response1);
    assert_eq!(b"+PONG\r\n", &response2);
    assert_eq!(b"+PONG\r\n", &response3);
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

#[tokio::test]
async fn get_and_set_value() {
    let addr = start_server().await;

    let mut stream = TcpStream::connect(addr).await.unwrap();

    stream
        .write_all(b"*3\r\n$3\r\nSET\r\n$3\r\nfoo\r\n$3\r\nbar\r\n")
        .await
        .unwrap();

    let mut response = [0; 5];
    stream.read(&mut response).await.unwrap();
    assert_eq!(b"+OK\r\n", &response);

    stream
        .write_all(b"*2\r\n$3\r\nGET\r\n$3\r\nfoo\r\n")
        .await
        .unwrap();

    let mut response = [0; 9];
    stream.read(&mut response).await.unwrap();
    assert_eq!(b"$3\r\nbar\r\n", &response);
}

#[tokio::test]
async fn get_nonexistent_value() {
    let addr = start_server().await;

    let mut stream = TcpStream::connect(addr).await.unwrap();

    stream
        .write_all(b"*2\r\n$3\r\nGET\r\n$3\r\nfoo\r\n")
        .await
        .unwrap();

    let mut response = [0; 5];
    stream.read(&mut response).await.unwrap();
    assert_eq!(b"$-1\r\n", &response);
}

async fn start_server() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move { server::run(listener, tokio::signal::ctrl_c()).await });

    addr
}
