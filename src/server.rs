use crate::{Command, Connection};
use tokio::net::{TcpListener, TcpStream};

#[derive(Debug)]
pub struct Listener {
    listener: TcpListener,
}

#[derive(Debug)]
pub struct Handler {
    connection: Connection,
}

impl Listener {
    async fn run(&mut self) -> crate::Result<()> {
        println!("accepting inbound connections");

        loop {
            let socket = self.accept().await?;

            let mut handler = Handler {
                connection: Connection::new(socket),
            };

            tokio::spawn(async move {
                if let Err(err) = handler.run().await {
                    let _ = anyhow::anyhow!("connection error: {}", err);
                }
            });
        }
    }

    async fn accept(&mut self) -> crate::Result<TcpStream> {
        loop {
            match self.listener.accept().await {
                Ok((socket, _)) => return Ok(socket),
                Err(err) => return Err(err.into()),
            }
        }
    }
}

impl Handler {
    async fn run(&mut self) -> crate::Result<()> {
        let maybe_frame = self.connection.read_frame().await?;
        let frame = match maybe_frame {
            Some(frame) => frame,
            None => return Ok(()),
        };
        let cmd = Command::from_frame(frame)?;
        cmd.apply(&mut self.connection).await?;
        Ok(())
    }
}

pub async fn run(listener: TcpListener) {
    let mut server = Listener { listener };
    let _ = server.run().await;
}
