mod network;

use crate::network::{
    CmLogin, CmServerInfo, PacketRegistry, ServerConfig, Session, SessionManager, WorkerPool,
};
use std::error::Error;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("===========================================");
    println!("     Knight Age Server - Starting...       ");
    println!("===========================================");

    // Create packet registry and register handlers
    let registry = create_packet_registry();
    let registry = Arc::new(registry);

    // Create server config
    let config = ServerConfig::new()
        .with_workers(4) // 4 worker threads for packet processing
        .with_bind_address("127.0.0.1:19129");

    // Create worker pool
    let worker_pool = WorkerPool::new(config.num_workers, registry.clone());
    let worker_sender = worker_pool.get_sender();

    // Create session manager
    let session_manager = Arc::new(SessionManager::new());

    // Start the game server
    start_game_server(config, session_manager, worker_sender).await?;

    Ok(())
}

/// Create and configure the packet registry with all handlers
fn create_packet_registry() -> PacketRegistry {
    let mut registry = PacketRegistry::new();

    // Register all packet handlers here
    // Similar to your Java handler registration
    registry.register(CmLogin);
    registry.register(CmServerInfo);

    // Add more handlers as needed:
    // registry.register(CmRegister);
    // registry.register(CmListChar);
    // registry.register(CmCreateChar);
    // registry.register(CmSelectChar);
    // registry.register(CmObjectMove);
    // registry.register(CmChatWorld);
    // ...

    println!(
        "[PacketRegistry] Total handlers registered: {}",
        registry.handler_count()
    );

    registry
}

async fn start_game_server(
    config: ServerConfig,
    session_manager: Arc<SessionManager>,
    worker_sender: tokio::sync::mpsc::Sender<network::WorkerTask>,
) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(&config.bind_address).await?;

    println!("[GameServer] Listening on {}", config.bind_address);
    println!("[GameServer] Worker threads: {}", config.num_workers);
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
                let ctx = session.get_context().clone();

                // Set worker sender for packet processing
                session.set_worker_sender(worker_sender.clone());

                // Register session
                let manager = session_manager.clone();
                manager.register(session_id, writer, ctx).await;

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
