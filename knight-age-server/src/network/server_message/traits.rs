//! Server Message Trait
//!
//! Trait này định nghĩa cách tạo packet gửi về client.

use crate::network::packet::writer::PacketWriter;

/// Trait cho server message (packet từ server → client)
///
/// # Cách implement
///
/// ```rust
/// pub struct SmWelcome {
///     message: String,
/// }
///
/// impl ServerMessage for SmWelcome {
///     fn command(&self) -> i8 { 37 }  // CMD 37 = INFO_FROM_SERVER
///     
///     fn write(&self, w: &mut PacketWriter) {
///         w.write_string(&self.message);
///     }
/// }
/// ```
pub trait ServerMessage: Send + Sync {
    /// CMD của message này
    fn command(&self) -> i8;

    /// Ghi data vào PacketWriter
    fn write(&self, writer: &mut PacketWriter);

    /// Build packet data hoàn chỉnh
    fn build(&self) -> Vec<u8> {
        let mut writer = PacketWriter::new();
        self.write(&mut writer);
        writer.into_inner()
    }

    /// Tên message (để debug)
    fn name(&self) -> &'static str {
        "ServerMessage"
    }
}
