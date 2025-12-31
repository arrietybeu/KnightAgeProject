use crate::network::context::PacketContext;
use crate::network::handler::{PacketHandler, PacketHandlerResult};
use crate::network::opcode::cmd;
use crate::network::packet::reader::PacketReader;
use crate::network::state::ConnectionState;
use async_trait::async_trait;
use std::io;
use std::io::*;

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
    pub character_id: i32,  // -1 = new login, otherwise reconnect
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
    fn parse_login_request(reader: &mut PacketReader) -> Result<LoginRequest> {
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
        let request = match Self::parse_login_request(reader) {
            Ok(req) => req,
            Err(e) => {
                eprintln!(
                    "[Session {}] Failed to parse login request: {}",
                    ctx.session_id, e
                );
                return Ok(PacketHandlerResult::Error(format!(
                    "Failed to parse login: {}",
                    e
                )));
            }
        };

        println!(
            "[Session {}] Login request:",
            ctx.session_id
        );
        println!("  - Username: {}", request.username);
        println!("  - Version: {}", request.version);
        println!("  - Device: {} (PC: {})", request.device, request.is_pc);
        println!("  - Zoom: {}, Area: {}", request.zoom_level, request.area);
        println!("  - Character ID: {}", request.character_id);
        println!("  - Package: {}", request.package_name);

        // Check if this is a new login or reconnect
        let is_new_login = request.character_id == -1;

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

        // TODO: Validate credentials with database
        // TODO: Check if account is already logged in
        // TODO: Send proper response packets

        if is_new_login {
            println!("[Session {}] New login for user: {}", ctx.session_id, request.username);
            
            // Send login success response
            // TODO: Implement proper response based on your server packets
            // For now, just send a simple response
            ctx.send_with(cmd::INFO_FROM_SERVER, |w| {
                w.write_string(&format!("Welcome {}!", request.username));
            })
            .await?;
        } else {
            println!(
                "[Session {}] Reconnect for user: {} (char_id: {})",
                ctx.session_id, request.username, request.character_id
            );
            
            // Handle reconnect
            // TODO: Restore character session
        }

        Ok(PacketHandlerResult::Ok)
    }

    fn name(&self) -> &'static str {
        "CmLogin"
    }
}
