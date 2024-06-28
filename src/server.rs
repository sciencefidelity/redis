use crate::{db, Command, Connection, Db, Shutdown};

use std::future::Future;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc};

#[derive(Debug)]
pub struct Listener {
    db_holder: db::DropGuard,
    listener: TcpListener,
    notify_shutdown: broadcast::Sender<()>,
    shutdown_complete_tx: mpsc::Sender<()>,
}

#[derive(Debug)]
pub struct Handler {
    db: Db,
    connection: Connection,
    shutdown: Shutdown,
    _shutdown_complete: mpsc::Sender<()>,
}

// TODO: implement send for `shutdown`
#[allow(clippy::future_not_send)]
pub async fn run(listener: TcpListener, shutdown: impl Future) {
    let (notify_shutdown, _) = broadcast::channel(1);
    let (shutdown_complete_tx, mut shutdown_complete_rx) = mpsc::channel(1);

    let server = Listener {
        listener,
        db_holder: db::DropGuard::new(),
        notify_shutdown,
        shutdown_complete_tx,
    };

    tokio::select! {
        res = server.run() => {
            if let Err(err) = res {
                let _ = anyhow::anyhow!("failed to accept, {}", err);
            }
        }
        _ = shutdown => {
            println!("shutting down");
        }
    }

    let Listener {
        shutdown_complete_tx,
        notify_shutdown,
        ..
    } = server;

    drop(notify_shutdown);
    drop(shutdown_complete_tx);

    let _ = shutdown_complete_rx.recv().await;
}

impl Listener {
    async fn run(&self) -> crate::Result<()> {
        println!("accepting inbound connections");

        loop {
            let socket = self.accept().await?;

            let mut handler = Handler {
                db: self.db_holder.db(),
                connection: Connection::new(socket),
                shutdown: Shutdown::new(self.notify_shutdown.subscribe()),
                _shutdown_complete: self.shutdown_complete_tx.clone(),
            };

            tokio::spawn(async move {
                if let Err(err) = handler.run().await {
                    let _ = anyhow::anyhow!("connection error: {}", err);
                }
            });
        }
    }

    async fn accept(&self) -> crate::Result<TcpStream> {
        match self.listener.accept().await {
            Ok((socket, _)) => Ok(socket),
            Err(err) => Err(err.into()),
        }
    }
}

impl Handler {
    async fn run(&mut self) -> crate::Result<()> {
        while !self.shutdown.is_shutdown() {
            let maybe_frame = tokio::select! {
                res = self.connection.read_frame() => res?,
                () = self.shutdown.recv() => {
                    return Ok(());
                }
            };

            let Some(frame) = maybe_frame else {
                return Ok(());
            };

            let cmd = Command::from_frame(frame)?;
            cmd.apply(&self.db, &mut self.connection).await?;
        }

        Ok(())
    }
}
