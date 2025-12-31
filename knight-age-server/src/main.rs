mod network;

use crate::network::{Session, SessionManager};
use std::error::Error;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("===========================================");
    println!("     Knight Age Server - Starting...       ");
    println!("===========================================");

    let session_manager = Arc::new(SessionManager::new());
    let game_server_task = start_game_server(session_manager);

    tokio::select! {
        result = game_server_task => {
            if let Err(e) = result {
                eprintln!("Game server error: {}", e);
            }
        }
    }

    Ok(())
}

async fn start_game_server(session_manager: Arc<SessionManager>) -> Result<(), Box<dyn Error>> {
    let addr = "127.0.0.1:19129";
    let listener = TcpListener::bind(addr).await?;

    println!("[GameServer] Listening on {}", addr);
    println!("[GameServer] Waiting for connections...");
    println!();

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                println!("[GameServer] New connection from {}", addr);

                let (read_half, write_half) = socket.into_split();
                let mut session = Session::new(read_half, write_half, addr);
                let session_id = session.get_id();
                let writer = session.get_writer();

                // Register session
                let manager = session_manager.clone();
                manager.register(session_id, writer).await;

                // Spawn session handler
                let manager_clone = session_manager.clone();
                tokio::spawn(async move {
                    if let Err(e) = session.run().await {
                        eprintln!("[Session {}] Error: {}", session_id, e);
                    }
                    manager_clone.unregister(session_id).await;
                });
            }
            Err(e) => {
                eprintln!("[GameServer] Failed to accept connection: {}", e);
            }
        }
    }
}
