use crate::network::connection::{
    create_shared_writer, ConnectionReader, ConnectionWriter, SharedWriter, CMD_KEY_EXCHANGE,
};
use crate::network::crypto::XorCipher;
use crate::network::packet::Packet;
use std::io;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc;

/// Global session ID counter
static SESSION_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Default key length for encryption
const DEFAULT_KEY_LENGTH: usize = 16;

/// Session state
#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    Connected,
    KeyExchanged,
    Authenticated,
    InGame,
    Disconnected,
}

/// Represents a client session
pub struct Session {
    pub id: u64,
    pub addr: SocketAddr,
    pub state: SessionState,
    reader: ConnectionReader,
    writer: SharedWriter,
    cipher: Option<XorCipher>,
    packet_sender: Option<mpsc::Sender<Packet>>,
}

impl Session {
    /// Create a new session from TCP stream halves
    pub fn new(
        read_half: OwnedReadHalf,
        write_half: OwnedWriteHalf,
        addr: SocketAddr,
    ) -> Self {
        let id = SESSION_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let reader = ConnectionReader::new(read_half);
        let writer = create_shared_writer(ConnectionWriter::new(write_half));

        Self {
            id,
            addr,
            state: SessionState::Connected,
            reader,
            writer,
            cipher: None,
            packet_sender: None,
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

    /// Get current state
    pub fn get_state(&self) -> &SessionState {
        &self.state
    }

    /// Set packet sender for async packet handling
    pub fn set_packet_sender(&mut self, sender: mpsc::Sender<Packet>) {
        self.packet_sender = Some(sender);
    }

    /// Get a clone of the shared writer
    pub fn get_writer(&self) -> SharedWriter {
        self.writer.clone()
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
            let mut writer = self.writer.lock().await;
            writer.write_key_exchange(&key_for_client).await?;
        }

        // Store cipher for encryption/decryption
        self.cipher = Some(cipher.clone());

        // Set cipher for reader and writer
        self.reader.set_cipher(cipher.clone());
        {
            let mut writer = self.writer.lock().await;
            writer.set_cipher(cipher);
        }

        self.state = SessionState::KeyExchanged;
        println!("[Session {}] Key exchange completed", self.id);

        Ok(())
    }

    /// Read next packet from client
    pub async fn read_packet(&mut self) -> io::Result<Packet> {
        self.reader.read_packet().await
    }

    /// Send a packet to client
    pub async fn send_packet(&self, packet: &Packet) -> io::Result<()> {
        let mut writer = self.writer.lock().await;
        writer.write_packet(packet).await
    }

    /// Send a packet with just command (no data)
    pub async fn send_cmd(&self, cmd: i8) -> io::Result<()> {
        self.send_packet(&Packet::new(cmd)).await
    }

    /// Main session loop - handles incoming packets
    pub async fn run(&mut self) -> io::Result<()> {
        println!("[Session {}] Started from {}", self.id, self.addr);

        loop {
            match self.read_packet().await {
                Ok(packet) => {
                    // Handle key exchange request
                    if packet.cmd == CMD_KEY_EXCHANGE && self.state == SessionState::Connected {
                        if let Err(e) = self.do_key_exchange().await {
                            eprintln!("[Session {}] Key exchange failed: {}", self.id, e);
                            break;
                        }
                        continue;
                    }

                    // Forward packet to handler if sender is set
                    if let Some(ref sender) = self.packet_sender {
                        if sender.send(packet.clone()).await.is_err() {
                            eprintln!("[Session {}] Packet handler disconnected", self.id);
                            break;
                        }
                    }

                    // Log received packet
                    println!(
                        "[Session {}] Received CMD: {} ({} bytes)",
                        self.id,
                        packet.cmd,
                        packet.data.len()
                    );
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

        self.state = SessionState::Disconnected;
        println!("[Session {}] Ended", self.id);
        Ok(())
    }
}

/// Session manager for handling multiple sessions
pub struct SessionManager {
    sessions: Arc<tokio::sync::RwLock<std::collections::HashMap<u64, SharedWriter>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Register a session
    pub async fn register(&self, session_id: u64, writer: SharedWriter) {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id, writer);
        println!("[SessionManager] Registered session {}", session_id);
    }

    /// Unregister a session
    pub async fn unregister(&self, session_id: u64) {
        let mut sessions = self.sessions.write().await;
        sessions.remove(&session_id);
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

    /// Get number of active sessions
    pub async fn count(&self) -> usize {
        self.sessions.read().await.len()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

