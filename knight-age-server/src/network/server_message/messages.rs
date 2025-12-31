//! Predefined Server Messages
//! 
//! This module contains common server message implementations.
//! Each struct implements the ServerMessage trait.
//! 
//! Naming convention: Sm = Server Message (vs Cm = Client Message)

use crate::network::opcode::cmd;
use crate::network::packet::writer::PacketWriter;
use crate::network::server_message::traits::ServerMessage;
use crate::network::server_message::service::RebuildItemData;

// ========== Login Messages ==========

pub struct SmLoginOk {
    pub map_table: Vec<i8>,
}

impl SmLoginOk {
    pub fn new(map_table: Vec<i8>) -> Self {
        Self { map_table }
    }
}

impl ServerMessage for SmLoginOk {
    fn command(&self) -> i8 {
        cmd::LOGIN
    }

    fn write(&self, w: &mut PacketWriter) {
        w.write_short(self.map_table.len() as i16);
        for &b in &self.map_table {
            w.write_i8(b);
        }
    }

    fn name(&self) -> &'static str {
        "SmLoginOk"
    }
}

/// Login failure message (CMD 2)
pub struct SmLoginFail {
    pub message: String,
    pub can_retry: bool,
}

impl SmLoginFail {
    pub fn new(message: impl Into<String>, can_retry: bool) -> Self {
        Self {
            message: message.into(),
            can_retry,
        }
    }
}

impl ServerMessage for SmLoginFail {
    fn command(&self) -> i8 {
        cmd::LOGIN_FAIL
    }

    fn write(&self, w: &mut PacketWriter) {
        w.write_string(&self.message);
        w.write_i8(if self.can_retry { 1 } else { 0 });
    }

    fn name(&self) -> &'static str {
        "SmLoginFail"
    }
}

// ========== Server Info Messages ==========

/// Info from server dialog (CMD 37)
pub struct SmInfoFromServer {
    pub message: String,
    pub link: String,
    pub msg_type: i8,
}

impl SmInfoFromServer {
    pub fn new(message: impl Into<String>, link: impl Into<String>, msg_type: i8) -> Self {
        Self {
            message: message.into(),
            link: link.into(),
            msg_type,
        }
    }

    /// Create a simple info message
    pub fn info(message: impl Into<String>) -> Self {
        Self::new(message, "", 0)
    }

    /// Create a message with download link
    pub fn with_link(message: impl Into<String>, link: impl Into<String>) -> Self {
        Self::new(message, link, 0)
    }
}

impl ServerMessage for SmInfoFromServer {
    fn command(&self) -> i8 {
        cmd::INFO_FROM_SERVER
    }

    fn write(&self, w: &mut PacketWriter) {
        w.write_string(&self.message);
        w.write_string(&self.link);
        w.write_i8(self.msg_type);
    }

    fn name(&self) -> &'static str {
        "SmInfoFromServer"
    }
}

/// Name server response (CMD 61)
pub struct SmNameServer {
    pub world_names: Vec<String>,
    pub quest_item_names: Vec<String>,
    pub material_ids: Vec<i16>,
    pub rebuild_data: Vec<RebuildItemData>,
}

impl SmNameServer {
    pub fn new() -> Self {
        Self {
            world_names: Vec::new(),
            quest_item_names: Vec::new(),
            material_ids: Vec::new(),
            rebuild_data: Vec::new(),
        }
    }

    pub fn with_world_names(mut self, names: Vec<String>) -> Self {
        self.world_names = names;
        self
    }

    pub fn with_quest_items(mut self, names: Vec<String>) -> Self {
        self.quest_item_names = names;
        self
    }

    pub fn with_materials(mut self, ids: Vec<i16>) -> Self {
        self.material_ids = ids;
        self
    }

    pub fn with_rebuild_data(mut self, data: Vec<RebuildItemData>) -> Self {
        self.rebuild_data = data;
        self
    }
}

impl Default for SmNameServer {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerMessage for SmNameServer {
    fn command(&self) -> i8 {
        cmd::NAME_SERVER
    }

    fn write(&self, w: &mut PacketWriter) {
        // World names (unsigned byte count)
        w.write_u8(self.world_names.len() as u8);
        for name in &self.world_names {
            w.write_string(name);
        }

        // Quest item names (signed byte count)
        w.write_i8(self.quest_item_names.len() as i8);
        for name in &self.quest_item_names {
            w.write_string(name);
        }

        // Material IDs (signed byte count)
        w.write_i8(self.material_ids.len() as i8);
        for &id in &self.material_ids {
            w.write_short(id);
        }

        // Rebuild data (signed byte count)
        w.write_i8(self.rebuild_data.len() as i8);
        for data in &self.rebuild_data {
            w.write_i8(data.level);
            w.write_int(data.price_coin);
            w.write_short(data.price_gold);
            for &v in &data.m_value {
                w.write_i8(v);
            }
        }
    }

    fn name(&self) -> &'static str {
        "SmNameServer"
    }
}

/// Delete RMS command (CMD 63)
pub struct SmDeleteRms {
    pub res_index: i8,
}

impl SmDeleteRms {
    pub fn new(res_index: i8) -> Self {
        Self { res_index }
    }
}

impl ServerMessage for SmDeleteRms {
    fn command(&self) -> i8 {
        cmd::DELETE_RMS
    }

    fn write(&self, w: &mut PacketWriter) {
        w.write_i8(self.res_index);
    }

    fn name(&self) -> &'static str {
        "SmDeleteRms"
    }
}

// ========== Chat Messages ==========

/// Chat popup message (CMD 27)
pub struct SmChatPopup {
    pub player_id: i16,
    pub message: String,
}

impl SmChatPopup {
    pub fn new(player_id: i16, message: impl Into<String>) -> Self {
        Self {
            player_id,
            message: message.into(),
        }
    }
}

impl ServerMessage for SmChatPopup {
    fn command(&self) -> i8 {
        cmd::CHAT_POPUP
    }

    fn write(&self, w: &mut PacketWriter) {
        w.write_short(self.player_id);
        w.write_string(&self.message);
    }

    fn name(&self) -> &'static str {
        "SmChatPopup"
    }
}

/// Chat tab message (CMD 34)
pub struct SmChatTab {
    pub tab_type: i8,
    pub sender: String,
    pub message: String,
}

impl SmChatTab {
    pub fn new(tab_type: i8, sender: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            tab_type,
            sender: sender.into(),
            message: message.into(),
        }
    }
}

impl ServerMessage for SmChatTab {
    fn command(&self) -> i8 {
        cmd::CHAT_TAB
    }

    fn write(&self, w: &mut PacketWriter) {
        w.write_i8(self.tab_type);
        w.write_string(&self.sender);
        w.write_string(&self.message);
    }

    fn name(&self) -> &'static str {
        "SmChatTab"
    }
}

// ========== Easy Info Messages ==========

/// Easy info message (CMD 53)
pub struct SmInfoEasy {
    pub message: String,
    pub info_type: i8, // 0 = addInfoChar, 1 = addInfoCharServer
}

impl SmInfoEasy {
    pub fn new(message: impl Into<String>, info_type: i8) -> Self {
        Self {
            message: message.into(),
            info_type,
        }
    }

    pub fn info_char(message: impl Into<String>) -> Self {
        Self::new(message, 0)
    }

    pub fn info_char_server(message: impl Into<String>) -> Self {
        Self::new(message, 1)
    }
}

impl ServerMessage for SmInfoEasy {
    fn command(&self) -> i8 {
        cmd::INFO_EASY_SERVER
    }

    fn write(&self, w: &mut PacketWriter) {
        w.write_string(&self.message);
        w.write_i8(self.info_type);
    }

    fn name(&self) -> &'static str {
        "SmInfoEasy"
    }
}

// ========== Player Messages ==========

/// Set experience (CMD 30)
pub struct SmSetExp {
    pub player_id: i16,
    pub percent: i16,
    pub exp_gained: i32,
}

impl SmSetExp {
    pub fn new(player_id: i16, percent: i16, exp_gained: i32) -> Self {
        Self {
            player_id,
            percent,
            exp_gained,
        }
    }
}

impl ServerMessage for SmSetExp {
    fn command(&self) -> i8 {
        cmd::SET_EXP
    }

    fn write(&self, w: &mut PacketWriter) {
        w.write_short(self.player_id);
        w.write_short(self.percent);
        w.write_int(self.exp_gained);
    }

    fn name(&self) -> &'static str {
        "SmSetExp"
    }
}

/// Level up notification (CMD 33)
pub struct SmLevelUp {
    pub player_id: i16,
    pub new_level: i8,
}

impl SmLevelUp {
    pub fn new(player_id: i16, new_level: i8) -> Self {
        Self { player_id, new_level }
    }
}

impl ServerMessage for SmLevelUp {
    fn command(&self) -> i8 {
        cmd::LEVEL_UP
    }

    fn write(&self, w: &mut PacketWriter) {
        w.write_short(self.player_id);
        w.write_i8(self.new_level);
    }

    fn name(&self) -> &'static str {
        "SmLevelUp"
    }
}

// ========== Data Messages ==========

/// Update data version (CMD -57)
pub struct SmUpdateData {
    pub version: i32,
}

impl SmUpdateData {
    pub fn new(version: i32) -> Self {
        Self { version }
    }
}

impl ServerMessage for SmUpdateData {
    fn command(&self) -> i8 {
        cmd::UPDATE_DATA
    }

    fn write(&self, w: &mut PacketWriter) {
        w.write_int(self.version);
    }

    fn name(&self) -> &'static str {
        "SmUpdateData"
    }
}

