use crate::network::connection::SharedWriter;
use crate::network::packet::writer::PacketWriter;
use crate::network::packet::Packet;
use crate::network::state::ConnectionState;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Device information from client
#[derive(Debug, Clone, Default)]
pub struct ClientDeviceInfo {
    pub type_client: i8,
    pub zoom_level: i8,
    pub screen_width: i32,
    pub screen_height: i32,
    pub is_qwerty: bool,
    pub is_touch: bool,
    pub platform_info: String,
    pub extra_info: Vec<u8>,
}

/// Session information
#[derive(Debug, Clone, Default)]
pub struct SessionInfo {
    pub version: String,
    pub is_logged_in: bool,
    pub account_id: Option<i64>,
    pub username: Option<String>,
    pub character_id: Option<i64>,
    pub device_info: ClientDeviceInfo,
}

/// Packet context - passed to every packet handler
/// Contains all information needed to process a packet
pub struct PacketContext {
    /// Session ID
    pub session_id: u64,
    /// Client address
    pub addr: SocketAddr,
    /// Current connection state
    state: Arc<RwLock<ConnectionState>>,
    /// Session information
    session_info: Arc<RwLock<SessionInfo>>,
    /// Writer for sending packets back
    writer: SharedWriter,
}

impl PacketContext {
    pub fn new(
        session_id: u64,
        addr: SocketAddr,
        writer: SharedWriter,
    ) -> Self {
        Self {
            session_id,
            addr,
            state: Arc::new(RwLock::new(ConnectionState::Connected)),
            session_info: Arc::new(RwLock::new(SessionInfo::default())),
            writer,
        }
    }

    /// Get current state
    pub async fn get_state(&self) -> ConnectionState {
        *self.state.read().await
    }

    /// Set connection state
    pub async fn set_state(&self, new_state: ConnectionState) {
        let mut state = self.state.write().await;
        println!(
            "[Session {}] State changed: {} -> {}",
            self.session_id, *state, new_state
        );
        *state = new_state;
    }

    /// Get session info (read-only)
    pub async fn get_session_info(&self) -> SessionInfo {
        self.session_info.read().await.clone()
    }

    /// Update session info
    pub async fn update_session_info<F>(&self, f: F)
    where
        F: FnOnce(&mut SessionInfo),
    {
        let mut info = self.session_info.write().await;
        f(&mut info);
    }

    /// Send a packet to client
    pub async fn send_packet(&self, packet: &Packet) -> std::io::Result<()> {
        let mut writer = self.writer.lock().await;
        writer.write_packet(packet).await
    }

    /// Send a packet with command and data
    pub async fn send(&self, cmd: i8, data: Vec<u8>) -> std::io::Result<()> {
        self.send_packet(&Packet::with_data(cmd, data)).await
    }

    /// Send a packet using PacketWriter
    pub async fn send_with<F>(&self, cmd: i8, f: F) -> std::io::Result<()>
    where
        F: FnOnce(&mut PacketWriter),
    {
        let mut writer = PacketWriter::new();
        f(&mut writer);
        self.send(cmd, writer.into_inner()).await
    }

    /// Close connection with a packet
    pub async fn close_with_packet(&self, packet: &Packet) -> std::io::Result<()> {
        self.send_packet(packet).await?;
        self.set_state(ConnectionState::Disconnecting).await;
        Ok(())
    }

    /// Get shared writer clone
    pub fn get_writer(&self) -> SharedWriter {
        self.writer.clone()
    }

    /// Clone state arc for sharing
    pub fn state_arc(&self) -> Arc<RwLock<ConnectionState>> {
        self.state.clone()
    }

    /// Clone session info arc for sharing
    pub fn session_info_arc(&self) -> Arc<RwLock<SessionInfo>> {
        self.session_info.clone()
    }
}

impl Clone for PacketContext {
    fn clone(&self) -> Self {
        Self {
            session_id: self.session_id,
            addr: self.addr,
            state: self.state.clone(),
            session_info: self.session_info.clone(),
            writer: self.writer.clone(),
        }
    }
}

