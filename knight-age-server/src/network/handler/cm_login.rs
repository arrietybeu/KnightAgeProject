use crate::network::context::PacketContext;
use crate::network::handler::{PacketHandler, PacketHandlerResult};
use crate::network::opcode::cmd;
use crate::network::packet::reader::PacketReader;
use crate::network::server_message::{PacketContextExt, SmDeleteRms, SmLoginFail, SmLoginOk};
use crate::network::state::ConnectionState;
use async_trait::async_trait;
use std::io;

/// Login request data from client
#[derive(Debug)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub version: String,
    pub client_protocol: String,
    pub pro: String,
    pub agent: String,
    pub zoom_level: i8,
    pub device: i8,
    pub character_id: i32, // -1 = new login, otherwise reconnect
    pub area: i8,
    pub is_pc: bool,
    pub resolution_index: i8,
    pub login_info_index: i8,
    pub reserved: i8,
    pub char_part_index: i16,
    pub package_name: String,
}

pub struct CmLogin;

impl CmLogin {
    fn parse_login_request(reader: &mut PacketReader) -> io::Result<LoginRequest> {
        Ok(LoginRequest {
            username: reader.read_string()?,
            password: reader.read_string()?,
            version: reader.read_string()?,
            client_protocol: reader.read_string()?,
            pro: reader.read_string()?,
            agent: reader.read_string()?,
            zoom_level: reader.read_i8()?,
            device: reader.read_i8()?,
            character_id: reader.read_int()?,
            area: reader.read_i8()?,
            is_pc: reader.read_i8()? == 1,
            resolution_index: reader.read_i8()?,
            login_info_index: reader.read_i8()?,
            reserved: reader.read_i8()?,
            char_part_index: reader.read_short()?,
            package_name: reader.read_string()?,
        })
    }

    /// Get default map table data
    /// This is placeholder data - in production, load from database/config
    fn get_map_table() -> Vec<i8> {
        // Map table format: each byte represents map configuration
        // In real implementation, this would be loaded from database
        vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10] // Placeholder
    }

    /// Validate user credentials
    /// TODO: Implement actual database validation
    fn validate_credentials(username: &str, _password: &str) -> bool {
        // For now, accept any login that's not empty
        // In production, verify against database
        !username.is_empty()
    }

    /// Check if user is already logged in
    /// TODO: Implement session tracking
    fn is_already_logged_in(_username: &str) -> bool {
        false
    }
}

#[async_trait]
impl PacketHandler for CmLogin {
    fn command(&self) -> i8 {
        cmd::LOGIN
    }

    fn valid_states(&self) -> Vec<ConnectionState> {
        vec![
            ConnectionState::KeyExchanged,
            ConnectionState::Authed,
            ConnectionState::InGame,
        ]
    }

    async fn handle(
        &self,
        reader: &mut PacketReader,
        ctx: &PacketContext,
    ) -> io::Result<PacketHandlerResult> {
        // Parse login request
        let request = match Self::parse_login_request(reader) {
            Ok(req) => req,
            Err(e) => {
                eprintln!(
                    "[Session {}] Failed to parse login request: {}",
                    ctx.session_id, e
                );
                
                // Send login failure
                let fail_msg = SmLoginFail::new("Dữ liệu đăng nhập không hợp lệ", false);
                ctx.send_message(&fail_msg).await?;
                
                return Ok(PacketHandlerResult::Error(format!(
                    "Failed to parse login: {}",
                    e
                )));
            }
        };

        // Log login attempt
        println!("[Session {}] Login request:", ctx.session_id);
        println!("  - Username: {}", request.username);
        println!("  - Version: {}", request.version);
        println!("  - Device: {} (PC: {})", request.device, request.is_pc);
        println!("  - Zoom: {}, Area: {}", request.zoom_level, request.area);
        println!("  - Character ID: {}", request.character_id);
        println!("  - Package: {}", request.package_name);

        // Check if this is initial login (placeholder data) or real login
        let is_placeholder_login = request.username == "1" && request.password == "1";
        if is_placeholder_login {
            println!("[Session {}] Placeholder login detected, waiting for real credentials", ctx.session_id);
            return Ok(PacketHandlerResult::Ok);
        }

        // Validate credentials
        if !Self::validate_credentials(&request.username, &request.password) {
            println!("[Session {}] Login failed: invalid credentials", ctx.session_id);
            
            let fail_msg = SmLoginFail::new("Tài khoản hoặc mật khẩu không đúng", true);
            ctx.send_message(&fail_msg).await?;
            
            return Ok(PacketHandlerResult::Ok);
        }

        // Check if already logged in
        if Self::is_already_logged_in(&request.username) {
            println!("[Session {}] Login failed: already logged in", ctx.session_id);
            
            let fail_msg = SmLoginFail::new("Tài khoản đang được đăng nhập", false);
            ctx.send_message(&fail_msg).await?;
            
            return Ok(PacketHandlerResult::Ok);
        }

        // Update session info
        ctx.update_session_info(|info| {
            info.username = Some(request.username.clone());
            info.version = request.version.clone();
            info.is_logged_in = true;
            info.device_info.zoom_level = request.zoom_level;
            info.device_info.type_client = request.device;
            info.device_info.platform_info = request.package_name.clone();
        })
        .await;

        // Change state to authenticated
        ctx.set_state(ConnectionState::Authed).await;

        let is_new_login = request.character_id == -1;

        if is_new_login {
            println!(
                "[Session {}] Login successful for user: {}",
                ctx.session_id, request.username
            );

            // 1. Send DELETE_RMS (CMD 63) - Tell client to clear cache
            let delete_rms = SmDeleteRms::new(request.resolution_index);
            ctx.send_message(&delete_rms).await?;
            println!("[Session {}] Sent DELETE_RMS (CMD 63)", ctx.session_id);

            // 2. Send LOGIN_OK (CMD 1) - Map table data
            // Note: In the original server, LOGIN_OK is sent after loading game data
            // For now, we send it here. In production, send after CATALOG_MONSTER, ITEM_TEMPLATE, etc.
            // TODO: Implement full game data loading sequence

        } else {
            println!(
                "[Session {}] Reconnect for user: {} (char_id: {})",
                ctx.session_id, request.username, request.character_id
            );

            // Handle reconnect
            // 1. Send DELETE_RMS
            let delete_rms = SmDeleteRms::new(request.resolution_index);
            ctx.send_message(&delete_rms).await?;

            // 2. Send LOGIN_OK with map table
            let login_ok = SmLoginOk::new(Self::get_map_table());
            ctx.send_message(&login_ok).await?;
            println!("[Session {}] Sent LOGIN_OK (CMD 1)", ctx.session_id);

            // TODO: Restore character session, send character info, etc.
        }

        Ok(PacketHandlerResult::Ok)
    }

    fn name(&self) -> &'static str {
        "CmLogin"
    }
}
