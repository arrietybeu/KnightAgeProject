use crate::network::context::PacketContext;
use crate::network::handler::{PacketHandler, PacketHandlerResult};
use crate::network::opcode::cmd;
use crate::network::packet::reader::PacketReader;
use crate::network::state::ConnectionState;
use async_trait::async_trait;
use std::io;

/// Server info request handler
/// CMD 61 (NAME_SERVER)
pub struct CmServerInfo;

#[async_trait]
impl PacketHandler for CmServerInfo {
    fn command(&self) -> i8 {
        cmd::NAME_SERVER
    }

    fn valid_states(&self) -> Vec<ConnectionState> {
        // Works in any authenticated state
        vec![
            ConnectionState::KeyExchanged,
            ConnectionState::Authed,
            ConnectionState::InGame,
        ]
    }

    async fn handle(
        &self,
        _reader: &mut PacketReader,
        ctx: &PacketContext,
    ) -> io::Result<PacketHandlerResult> {
        println!("[Session {}] Server info request", ctx.session_id);

        // Send server info response
        ctx.send_with(cmd::NAME_SERVER, |w| {
            w.write_string("Knight Age Server");
            w.write_string("v1.0.0");
            w.write_i8(1); // Server status: 1 = online
        })
        .await?;

        Ok(PacketHandlerResult::Ok)
    }

    fn name(&self) -> &'static str {
        "CmServerInfo"
    }
}

