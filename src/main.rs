use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn handle_connection(mut stream: TcpStream) {
    let mut buf = [0; 512];
    let wbuf = b"+PONG\r\n";

    loop {
        match stream.read(&mut buf) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    break;
                }
                let _ = stream.write_all(wbuf);
            }
            Err(_) => {}
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let _ = std::thread::spawn(move || handle_connection(stream));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
