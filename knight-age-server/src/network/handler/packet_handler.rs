use crate::network::connection::SharedWriter;
use crate::network::opcode::{cmd, get_cmd_name};
use crate::network::packet::reader::PacketReader;
use crate::network::packet::writer::PacketWriter;
use crate::network::packet::Packet;
use std::io;

/// Packet handler trait - implement this for each command type
pub trait PacketHandler: Send + Sync {
    fn handle(&self, packet: &Packet, writer: SharedWriter) -> impl std::future::Future<Output = io::Result<()>> + Send;
}

/// Default packet handler with basic routing
pub struct DefaultPacketHandler;

impl DefaultPacketHandler {
    pub fn new() -> Self {
        Self
    }

    /// Handle incoming packet
    pub async fn handle_packet(&self, packet: &Packet, writer: SharedWriter) -> io::Result<()> {
        let cmd_name = get_cmd_name(packet.cmd);
        println!("[Handler] Processing {} (cmd: {})", cmd_name, packet.cmd);

        match packet.cmd {
            cmd::LOGIN => self.handle_login(packet, writer).await,
            cmd::REGISTER => self.handle_register(packet, writer).await,
            cmd::LIST_CHAR => self.handle_list_char(packet, writer).await,
            cmd::SELECT_CHAR => self.handle_select_char(packet, writer).await,
            cmd::CREATE_CHAR => self.handle_create_char(packet, writer).await,
            cmd::OBJECT_MOVE => self.handle_object_move(packet, writer).await,
            cmd::CHAT_WORLD => self.handle_chat_world(packet, writer).await,
            _ => {
                println!("[Handler] Unhandled command: {} ({})", cmd_name, packet.cmd);
                Ok(())
            }
        }
    }

    /// Handle LOGIN command
    async fn handle_login(&self, packet: &Packet, writer: SharedWriter) -> io::Result<()> {
        let mut reader = PacketReader::new(packet.data.clone());
        
        // Read login data from client
        // Format: username (string), password (string), ...
        let username = reader.read_string().unwrap_or_default();
        let password = reader.read_string().unwrap_or_default();
        
        println!("[Handler] Login attempt: user={}, pass=***", username);
        
        // TODO: Validate credentials
        // For now, just accept any login
        
        // Send response
        let mut response = PacketWriter::new();
        response.write_i8(1); // Success
        response.write_string("Login successful");
        
        let response_packet = Packet::with_data(cmd::LOGIN, response.into_inner());
        
        let mut w = writer.lock().await;
        w.write_packet(&response_packet).await
    }

    /// Handle REGISTER command
    async fn handle_register(&self, packet: &Packet, writer: SharedWriter) -> io::Result<()> {
        let mut reader = PacketReader::new(packet.data.clone());
        
        let username = reader.read_string().unwrap_or_default();
        let password = reader.read_string().unwrap_or_default();
        
        println!("[Handler] Register: user={}", username);
        
        // TODO: Create account in database
        
        // Send response
        let mut response = PacketWriter::new();
        response.write_i8(1); // Success
        response.write_string("Registration successful");
        
        let response_packet = Packet::with_data(cmd::REGISTER, response.into_inner());
        
        let mut w = writer.lock().await;
        w.write_packet(&response_packet).await
    }

    /// Handle LIST_CHAR command
    async fn handle_list_char(&self, _packet: &Packet, writer: SharedWriter) -> io::Result<()> {
        println!("[Handler] List characters request");
        
        // TODO: Fetch characters from database
        
        // Send response with empty character list for now
        let mut response = PacketWriter::new();
        response.write_i8(0); // Number of characters
        
        let response_packet = Packet::with_data(cmd::LIST_CHAR, response.into_inner());
        
        let mut w = writer.lock().await;
        w.write_packet(&response_packet).await
    }

    /// Handle SELECT_CHAR command
    async fn handle_select_char(&self, packet: &Packet, writer: SharedWriter) -> io::Result<()> {
        let mut reader = PacketReader::new(packet.data.clone());
        let char_index = reader.read_i8().unwrap_or(0);
        
        println!("[Handler] Select character: index={}", char_index);
        
        // TODO: Load character data
        
        Ok(())
    }

    /// Handle CREATE_CHAR command
    async fn handle_create_char(&self, packet: &Packet, writer: SharedWriter) -> io::Result<()> {
        let mut reader = PacketReader::new(packet.data.clone());
        let char_name = reader.read_string().unwrap_or_default();
        let char_class = reader.read_i8().unwrap_or(0);
        
        println!("[Handler] Create character: name={}, class={}", char_name, char_class);
        
        // TODO: Create character in database
        
        // Send response
        let mut response = PacketWriter::new();
        response.write_i8(1); // Success
        
        let response_packet = Packet::with_data(cmd::CREATE_CHAR, response.into_inner());
        
        let mut w = writer.lock().await;
        w.write_packet(&response_packet).await
    }

    /// Handle OBJECT_MOVE command
    async fn handle_object_move(&self, packet: &Packet, _writer: SharedWriter) -> io::Result<()> {
        let mut reader = PacketReader::new(packet.data.clone());
        
        // Read movement data
        let x = reader.read_short().unwrap_or(0);
        let y = reader.read_short().unwrap_or(0);
        
        println!("[Handler] Object move: x={}, y={}", x, y);
        
        // TODO: Update position and broadcast to nearby players
        
        Ok(())
    }

    /// Handle CHAT_WORLD command
    async fn handle_chat_world(&self, packet: &Packet, _writer: SharedWriter) -> io::Result<()> {
        let mut reader = PacketReader::new(packet.data.clone());
        let message = reader.read_string().unwrap_or_default();
        
        println!("[Handler] World chat: {}", message);
        
        // TODO: Broadcast to all players
        
        Ok(())
    }
}

impl Default for DefaultPacketHandler {
    fn default() -> Self {
        Self::new()
    }
}

