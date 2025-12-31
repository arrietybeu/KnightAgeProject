//! Server Service - Helper for sending common packets
//! 
//! This is similar to Java's GlobalService pattern.
//! Provides convenient methods for sending common server packets.

use crate::network::context::PacketContext;
use crate::network::opcode::cmd;
use crate::network::packet::writer::PacketWriter;
use crate::network::packet::Packet;
use crate::network::server_message::traits::ServerMessage;
use std::io;

/// ServerService provides convenient methods for sending server packets
/// 
/// # Usage
/// 
/// ```rust
/// let service = ServerService::new(&ctx);
/// service.send_info_from_server("Welcome!").await?;
/// service.send_login_ok(&map_table).await?;
/// ```
pub struct ServerService<'a> {
    ctx: &'a PacketContext,
}

impl<'a> ServerService<'a> {
    /// Create a new ServerService from a PacketContext
    pub fn new(ctx: &'a PacketContext) -> Self {
        Self { ctx }
    }

    /// Get the underlying context
    pub fn context(&self) -> &PacketContext {
        self.ctx
    }

    // ========== Low-level Send Methods ==========

    /// Send a raw packet
    pub async fn send_packet(&self, packet: &Packet) -> io::Result<()> {
        self.ctx.send_packet(packet).await
    }

    /// Send a ServerMessage
    pub async fn send_message<M: ServerMessage>(&self, msg: &M) -> io::Result<()> {
        let data = msg.build();
        self.ctx.send(msg.command(), data).await
    }

    /// Send with builder function
    pub async fn send_with<F>(&self, cmd: i8, f: F) -> io::Result<()>
    where
        F: FnOnce(&mut PacketWriter),
    {
        self.ctx.send_with(cmd, f).await
    }

    /// Send command with no data
    pub async fn send_cmd(&self, cmd: i8) -> io::Result<()> {
        self.ctx.send(cmd, Vec::new()).await
    }

    // ========== Login Related ==========

    /// Send login success response (CMD 1)
    /// Includes the map table data
    pub async fn send_login_ok(&self, map_table: &[i8]) -> io::Result<()> {
        self.send_with(cmd::LOGIN, |w| {
            w.write_short(map_table.len() as i16);
            for &b in map_table {
                w.write_i8(b);
            }
        }).await
    }

    /// Send login failure (CMD 2)
    pub async fn send_login_fail(&self, message: &str, can_retry: bool) -> io::Result<()> {
        self.send_with(cmd::LOGIN_FAIL, |w| {
            w.write_string(message);
            w.write_i8(if can_retry { 1 } else { 0 });
        }).await
    }

    // ========== Server Info ==========

    /// Send info from server dialog (CMD 37)
    pub async fn send_info_from_server(&self, message: &str) -> io::Result<()> {
        self.send_info_from_server_full(message, "", 0).await
    }

    /// Send info from server with download link (CMD 37)
    pub async fn send_info_from_server_full(
        &self,
        message: &str,
        link: &str,
        msg_type: i8,
    ) -> io::Result<()> {
        self.send_with(cmd::INFO_FROM_SERVER, |w| {
            w.write_string(message);
            w.write_string(link);
            w.write_i8(msg_type);
        }).await
    }

    /// Send name server data (CMD 61)
    pub async fn send_name_server(
        &self,
        world_names: &[&str],
        quest_item_names: &[&str],
        material_ids: &[i16],
        rebuild_data: &[RebuildItemData],
    ) -> io::Result<()> {
        self.send_with(cmd::NAME_SERVER, |w| {
            // World names (unsigned byte count)
            w.write_u8(world_names.len() as u8);
            for name in world_names {
                w.write_string(name);
            }

            // Quest item names (signed byte count)
            w.write_i8(quest_item_names.len() as i8);
            for name in quest_item_names {
                w.write_string(name);
            }

            // Material IDs (signed byte count)
            w.write_i8(material_ids.len() as i8);
            for &id in material_ids {
                w.write_short(id);
            }

            // Rebuild data (signed byte count)
            w.write_i8(rebuild_data.len() as i8);
            for data in rebuild_data {
                w.write_i8(data.level);
                w.write_int(data.price_coin);
                w.write_short(data.price_gold);
                for &v in &data.m_value {
                    w.write_i8(v);
                }
            }
        }).await
    }

    /// Send delete RMS command (CMD 63)
    pub async fn send_delete_rms(&self, res_index: i8) -> io::Result<()> {
        self.send_with(cmd::DELETE_RMS, |w| {
            w.write_i8(res_index);
        }).await
    }

    // ========== Chat ==========

    /// Send chat popup message (CMD 27)
    pub async fn send_chat_popup(&self, player_id: i16, message: &str) -> io::Result<()> {
        self.send_with(cmd::CHAT_POPUP, |w| {
            w.write_short(player_id);
            w.write_string(message);
        }).await
    }

    /// Send chat tab message (CMD 34)
    pub async fn send_chat_tab(&self, tab_type: i8, sender: &str, message: &str) -> io::Result<()> {
        self.send_with(cmd::CHAT_TAB, |w| {
            w.write_i8(tab_type);
            w.write_string(sender);
            w.write_string(message);
        }).await
    }

    // ========== Player Info ==========

    /// Send experience update (CMD 30)
    pub async fn send_set_exp(&self, player_id: i16, percent: i16, exp_gained: i32) -> io::Result<()> {
        self.send_with(cmd::SET_EXP, |w| {
            w.write_short(player_id);
            w.write_short(percent);
            w.write_int(exp_gained);
        }).await
    }

    /// Send level up (CMD 33)
    pub async fn send_level_up(&self, player_id: i16, new_level: i8) -> io::Result<()> {
        self.send_with(cmd::LEVEL_UP, |w| {
            w.write_short(player_id);
            w.write_i8(new_level);
        }).await
    }

    // ========== Easy Info ==========

    /// Send simple info message (CMD 53)
    /// type: 0 = addInfoChar, 1 = addInfoCharServer
    pub async fn send_info_easy(&self, message: &str, info_type: i8) -> io::Result<()> {
        self.send_with(cmd::INFO_EASY_SERVER, |w| {
            w.write_string(message);
            w.write_i8(info_type);
        }).await
    }

    /// Send info char message
    pub async fn send_info_char(&self, message: &str) -> io::Result<()> {
        self.send_info_easy(message, 0).await
    }

    /// Send info char server message
    pub async fn send_info_char_server(&self, message: &str) -> io::Result<()> {
        self.send_info_easy(message, 1).await
    }
}

/// Data structure for rebuild item configuration
#[derive(Debug, Clone, Default)]
pub struct RebuildItemData {
    pub level: i8,
    pub price_coin: i32,
    pub price_gold: i16,
    pub m_value: [i8; 4],
}

impl RebuildItemData {
    pub fn new(level: i8, price_coin: i32, price_gold: i16, m_value: [i8; 4]) -> Self {
        Self {
            level,
            price_coin,
            price_gold,
            m_value,
        }
    }
}

// ========== Extension trait for PacketContext ==========

/// Extension trait to add send_message to PacketContext
#[async_trait::async_trait]
pub trait PacketContextExt {
    /// Send a ServerMessage
    async fn send_message<M: ServerMessage>(&self, msg: &M) -> io::Result<()>;
    
    /// Get a ServerService for this context
    fn service(&self) -> ServerService<'_>;
}

#[async_trait::async_trait]
impl PacketContextExt for PacketContext {
    async fn send_message<M: ServerMessage>(&self, msg: &M) -> io::Result<()> {
        let data = msg.build();
        self.send(msg.command(), data).await
    }
    
    fn service(&self) -> ServerService<'_> {
        ServerService::new(self)
    }
}

