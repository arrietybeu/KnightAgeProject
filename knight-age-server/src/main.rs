mod network;

use std::error::Error;
use tokio::sync::mpsc;
use crate::network::packet::Packet;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    starting_networking().await;
    Ok(())
}

async fn starting_networking() {
    let listener = TcpListener::bind(("127.0.0.1", 19129)).await.expect("Failed to bind");

    loop {
        match listener.accept().await {
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
                continue;
            }
            Ok((socket, addr)) => {
                let (read_half, write_half) = socket.into_split();

                let (tx, rx): (mpsc::Sender<Packet>, mpsc::Receiver<Packet>) = mpsc::channel(100);

                let(hehe, hihi ): (i16, i16) = (5, 10);

                tokio::spawn(async move {
                    // connection_reader_loop(conn_reader, tx).await;
                });

                tokio::spawn(async move {
                    // let mut session = Session::new(rx, conn_writer);
                    // session.run().await;
                });

            }
        }
    }
}
