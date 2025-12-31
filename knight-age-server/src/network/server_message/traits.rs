//! Server Message Traits

use crate::network::packet::writer::PacketWriter;

/// Trait for server messages (packets sent from server to client)
/// 
/// Implement this trait to create structured, reusable server packets.
/// 
/// # Example
/// 
/// ```rust
/// pub struct SmWelcome {
///     message: String,
/// }
/// 
/// impl ServerMessage for SmWelcome {
///     fn command(&self) -> i8 { 37 }
///     
///     fn write(&self, w: &mut PacketWriter) {
///         w.write_string(&self.message);
///     }
/// }
/// ```
pub trait ServerMessage: Send + Sync {
    /// Get the command ID for this message
    fn command(&self) -> i8;
    
    /// Write packet data to the writer
    fn write(&self, writer: &mut PacketWriter);
    
    /// Build the complete packet data
    fn build(&self) -> Vec<u8> {
        let mut writer = PacketWriter::new();
        self.write(&mut writer);
        writer.into_inner()
    }
    
    /// Get message name for debugging
    fn name(&self) -> &'static str {
        "ServerMessage"
    }
}

/// Extension trait for optional server message features
pub trait ServerMessageExt: ServerMessage {
    /// Check if this is a priority message (should be sent immediately)
    fn is_priority(&self) -> bool {
        false
    }
    
    /// Get estimated packet size for buffer allocation
    fn estimated_size(&self) -> usize {
        1024
    }
}

// Blanket implementation
impl<T: ServerMessage> ServerMessageExt for T {}

