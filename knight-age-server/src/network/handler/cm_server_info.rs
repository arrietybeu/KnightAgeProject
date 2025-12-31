use crate::network::context::PacketContext;
use crate::network::handler::{PacketHandler, PacketHandlerResult};
use crate::network::opcode::cmd;
use crate::network::packet::reader::PacketReader;
use crate::network::server_message::{
    PacketContextExt, RebuildItemData, SmInfoFromServer, SmNameServer,
};
use crate::network::state::ConnectionState;
use async_trait::async_trait;
use std::io;

pub struct CmServerInfo;

impl CmServerInfo {
    fn get_world_names() -> Vec<String> {
        vec![
            "Làng Sói".to_string(),
        ]
    }

    fn get_quest_item_names() -> Vec<String> {
        vec![
            "Thuốc hồi máu".to_string(),
        ]
    }

    fn get_material_ids() -> Vec<i16> {
        vec![1001, 1002, 1003, 1004, 1005]
    }

    fn get_rebuild_data() -> Vec<RebuildItemData> {
        vec![
            RebuildItemData::new(1, 1000, 10, [1, 2, 3, 4]),
            RebuildItemData::new(5, 5000, 50, [2, 4, 6, 8]),
            RebuildItemData::new(10, 10000, 100, [5, 10, 15, 20]),
        ]
    }
}

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
        println!("[Session {}] Server info request (CMD 61)", ctx.session_id);

        let welcome_msg = SmInfoFromServer::info("Chào mừng bạn đến với Knight Age!");
        ctx.send_message(&welcome_msg).await?;
        println!("[Session {}] Sent INFO_FROM_SERVER (CMD 37)", ctx.session_id);

        // 2. Send NAME_SERVER response (CMD 61) - Full server data
        let name_server_msg = SmNameServer::new()
            .with_world_names(Self::get_world_names())
            .with_quest_items(Self::get_quest_item_names())
            .with_materials(Self::get_material_ids())
            .with_rebuild_data(Self::get_rebuild_data());
        
        ctx.send_message(&name_server_msg).await?;
        println!("[Session {}] Sent NAME_SERVER response (CMD 61)", ctx.session_id);

        Ok(PacketHandlerResult::Ok)
    }

    fn name(&self) -> &'static str {
        "CmServerInfo"
    }
}

