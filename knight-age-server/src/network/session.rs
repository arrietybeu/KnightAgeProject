use crate::network::connection::{
    create_shared_writer, ConnectionReader, ConnectionWriter, SharedWriter, CMD_KEY_EXCHANGE,
};
use crate::network::context::PacketContext;
use crate::network::crypto::XorCipher;
use crate::network::packet::Packet;
use crate::network::state::ConnectionState;
use crate::network::worker::WorkerTask;
use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::{mpsc, RwLock};

/// Global session ID counter
static SESSION_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Default key length for encryption
const DEFAULT_KEY_LENGTH: usize = 16;

/// Represents a client session
pub struct Session {
    pub id: u64,
    pub addr: SocketAddr,
    reader: ConnectionReader,
    ctx: PacketContext,
    worker_sender: Option<mpsc::Sender<WorkerTask>>,
}

impl Session {
    /// Create a new session from TCP stream halves
    pub fn new(read_half: OwnedReadHalf, write_half: OwnedWriteHalf, addr: SocketAddr) -> Self {
        let id = SESSION_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        let reader = ConnectionReader::new(read_half);
        let writer = create_shared_writer(ConnectionWriter::new(write_half));
        let ctx = PacketContext::new(id, addr, writer);

        Self {
            id,
            addr,
            reader,
            ctx,
            worker_sender: None,
        }
    }

    /// Get session ID
    pub fn get_id(&self) -> u64 {
        self.id
    }

    /// Get client address
    pub fn get_addr(&self) -> SocketAddr {
        self.addr
    }

    /// Get packet context
    pub fn get_context(&self) -> &PacketContext {
        &self.ctx
    }

    /// Get a clone of the shared writer
    pub fn get_writer(&self) -> SharedWriter {
        self.ctx.get_writer()
    }

    /// Set worker sender for packet processing
    pub fn set_worker_sender(&mut self, sender: mpsc::Sender<WorkerTask>) {
        self.worker_sender = Some(sender);
    }

    /// Perform key exchange with client
    pub async fn do_key_exchange(&mut self) -> io::Result<()> {
        // Generate random key
        let key = XorCipher::generate_key(DEFAULT_KEY_LENGTH);
        let cipher = XorCipher::new(key.clone());

        // Get key data to send to client (with inverse transformation)
        let key_for_client = cipher.get_key_for_client();

        // Send key exchange packet
        {
            let writer_arc = self.ctx.get_writer();
            let mut writer = writer_arc.lock().await;
            writer.write_key_exchange(&key_for_client).await?;
        }

        // Set cipher for reader and writer
        self.reader.set_cipher(cipher.clone());
        {
            let writer_arc = self.ctx.get_writer();
            let mut writer = writer_arc.lock().await;
            writer.set_cipher(cipher);
        }

        self.ctx.set_state(ConnectionState::KeyExchanged).await;
        println!("[Session {}] Key exchange completed", self.id);

        Ok(())
    }

    /// Read next packet from client
    pub async fn read_packet(&mut self) -> io::Result<Packet> {
        self.reader.read_packet().await
    }

    /// Main session loop - handles incoming packets
    pub async fn run(&mut self) -> io::Result<()> {
        println!("[Session {}] Started from {}", self.id, self.addr);

        loop {
            // Check if disconnecting
            if self.ctx.get_state().await == ConnectionState::Disconnecting {
                println!("[Session {}] Disconnecting...", self.id);
                break;
            }

            match self.read_packet().await {
                Ok(packet) => {
                    // Handle key exchange request
                    if packet.cmd == CMD_KEY_EXCHANGE
                        && self.ctx.get_state().await == ConnectionState::Connected
                    {
                        if let Err(e) = self.do_key_exchange().await {
                            eprintln!("[Session {}] Key exchange failed: {}", self.id, e);
                            break;
                        }
                        continue;
                    }

                    // Forward packet to worker pool
                    if let Some(ref sender) = self.worker_sender {
                        let task = WorkerTask {
                            packet,
                            ctx: self.ctx.clone(),
                        };
                        if sender.send(task).await.is_err() {
                            eprintln!("[Session {}] Worker pool disconnected", self.id);
                            break;
                        }
                    } else {
                        // No worker, just log
                        println!(
                            "[Session {}] Received CMD: {} ({} bytes) - no worker attached",
                            self.id,
                            packet.cmd,
                            packet.data.len()
                        );
                    }
                }
                Err(e) => {
                    if e.kind() == io::ErrorKind::UnexpectedEof {
                        println!("[Session {}] Client disconnected", self.id);
                    } else {
                        eprintln!("[Session {}] Read error: {}", self.id, e);
                    }
                    break;
                }
            }
        }

        self.ctx.set_state(ConnectionState::Disconnecting).await;
        println!("[Session {}] Ended", self.id);
        Ok(())
    }
}

/// Session manager for handling multiple sessions
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<u64, SharedWriter>>>,
    contexts: Arc<RwLock<HashMap<u64, PacketContext>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            contexts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a session
    pub async fn register(&self, session_id: u64, writer: SharedWriter, ctx: PacketContext) {
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id, writer);
        }
        {
            let mut contexts = self.contexts.write().await;
            contexts.insert(session_id, ctx);
        }
        println!("[SessionManager] Registered session {}", session_id);
    }

    /// Unregister a session
    pub async fn unregister(&self, session_id: u64) {
        {
            let mut sessions = self.sessions.write().await;
            sessions.remove(&session_id);
        }
        {
            let mut contexts = self.contexts.write().await;
            contexts.remove(&session_id);
        }
        println!("[SessionManager] Unregistered session {}", session_id);
    }

    /// Send packet to specific session
    pub async fn send_to(&self, session_id: u64, packet: &Packet) -> io::Result<()> {
        let sessions = self.sessions.read().await;
        if let Some(writer) = sessions.get(&session_id) {
            let mut writer = writer.lock().await;
            writer.write_packet(packet).await
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Session {} not found", session_id),
            ))
        }
    }

    /// Broadcast packet to all sessions
    pub async fn broadcast(&self, packet: &Packet) {
        let sessions = self.sessions.read().await;
        for (id, writer) in sessions.iter() {
            let mut writer = writer.lock().await;
            if let Err(e) = writer.write_packet(packet).await {
                eprintln!("[SessionManager] Failed to send to session {}: {}", id, e);
            }
        }
    }

    /// Broadcast to sessions matching a filter
    pub async fn broadcast_if<F>(&self, packet: &Packet, filter: F)
    where
        F: Fn(&PacketContext) -> bool,
    {
        let sessions = self.sessions.read().await;
        let contexts = self.contexts.read().await;

        for (id, writer) in sessions.iter() {
            if let Some(ctx) = contexts.get(id) {
                if filter(ctx) {
                    let mut writer = writer.lock().await;
                    if let Err(e) = writer.write_packet(packet).await {
                        eprintln!("[SessionManager] Failed to send to session {}: {}", id, e);
                    }
                }
            }
        }
    }

    /// Get number of active sessions
    pub async fn count(&self) -> usize {
        self.sessions.read().await.len()
    }

    /// Get context for a session
    pub async fn get_context(&self, session_id: u64) -> Option<PacketContext> {
        self.contexts.read().await.get(&session_id).cloned()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
