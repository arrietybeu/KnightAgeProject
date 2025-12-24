mod network;

use std::error::Error;
use tokio::sync::mpsc;
use crate::network::packet::Packet;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 19129)).await?;

    loop {
        let (socket, addr) = listener.accept().await?;

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
