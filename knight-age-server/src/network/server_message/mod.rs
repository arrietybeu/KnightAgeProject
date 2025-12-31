//! Server Message Module - Gửi packet từ Server → Client
//!
//! Module này cung cấp cách đơn giản để gửi packet về client.
//!
//! # Cách sử dụng
//!
//! ```rust
//! // Cách 1: Dùng struct có sẵn (khuyên dùng)
//! let msg = SmInfoFromServer::info("Xin chào!");
//! ctx.send_message(&msg).await?;
//!
//! // Cách 2: Dùng ctx.send_with trực tiếp
//! ctx.send_with(cmd::INFO_FROM_SERVER, |w| {
//!     w.write_string("Xin chào!");
//!     w.write_string("");  // link
//!     w.write_i8(0);       // type
//! }).await?;
//! ```

mod traits;
mod messages;

pub use traits::ServerMessage;
pub use messages::*;

// Re-export RebuildItemData from messages
pub use messages::RebuildItemData;

// ========== Extension để dùng ctx.send_message() ==========

use crate::network::context::PacketContext;
use std::io;

/// Extension trait - thêm method send_message vào PacketContext
#[async_trait::async_trait]
pub trait PacketContextExt {
    /// Gửi một ServerMessage về client
    async fn send_message<M: ServerMessage>(&self, msg: &M) -> io::Result<()>;
}

#[async_trait::async_trait]
impl PacketContextExt for PacketContext {
    async fn send_message<M: ServerMessage>(&self, msg: &M) -> io::Result<()> {
        let data = msg.build();
        self.send(msg.command(), data).await
    }
}
