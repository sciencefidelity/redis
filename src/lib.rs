use std::future::Future;
use std::io;
use tokio::net::{TcpListener, TcpStream};

pub struct Redis;

impl Redis {
    pub async fn process(stream: TcpStream) {
        const PROTO_MAX_BULK_LEN: usize = 512;
        let mut buf = [0; PROTO_MAX_BULK_LEN];
        let wbuf = b"+PONG\r\n";

        loop {
            stream
                .readable()
                .await
                .expect("Failed to get readable status");

            match stream.try_read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    println!("received {} bytes", n);
                    println!("data: {:?}", &buf[..n]);
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    println!("Unknown error! {:?}", e);
                    return;
                }
            }
            stream
                .writable()
                .await
                .expect("Failed to get writable status on stream");
            let _ = stream.try_write(wbuf);
        }
    }
}

pub async fn run(listener: TcpListener, _shutdown: impl Future) {
    loop {
        let (socket, _) = listener.accept().await.unwrap();

        tokio::spawn(async move {
            Redis::process(socket).await;
        });
    }
}
