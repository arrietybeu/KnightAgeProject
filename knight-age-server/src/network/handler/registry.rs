use crate::network::context::PacketContext;
use crate::network::handler::{PacketHandler, PacketHandlerResult};
use crate::network::opcode::get_cmd_name;
use crate::network::packet::reader::PacketReader;
use crate::network::packet::Packet;
use std::collections::HashMap;
use std::io;
use std::sync::Arc;

/// Registry for packet handlers
/// Maps command ID -> handler
pub struct PacketRegistry {
    handlers: HashMap<i8, Arc<dyn PacketHandler>>,
}

impl PacketRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a packet handler
    pub fn register<H: PacketHandler + 'static>(&mut self, handler: H) {
        let cmd = handler.command();
        let name = handler.name();
        self.handlers.insert(cmd, Arc::new(handler));
        println!(
            "[PacketRegistry] Registered handler: {} (cmd: {} / {})",
            name,
            cmd,
            get_cmd_name(cmd)
        );
    }

    /// Get handler for a command
    pub fn get_handler(&self, cmd: i8) -> Option<Arc<dyn PacketHandler>> {
        self.handlers.get(&cmd).cloned()
    }

    /// Process a packet
    pub async fn handle_packet(
        &self,
        packet: &Packet,
        ctx: &PacketContext,
    ) -> io::Result<PacketHandlerResult> {
        let cmd = packet.cmd;
        let cmd_name = get_cmd_name(cmd);

        // Get handler
        let handler = match self.get_handler(cmd) {
            Some(h) => h,
            None => {
                println!(
                    "[Session {}] No handler for {} (cmd: {})",
                    ctx.session_id, cmd_name, cmd
                );
                return Ok(PacketHandlerResult::Ok);
            }
        };

        // Validate state
        let current_state = ctx.get_state().await;
        if !handler.is_valid_state(current_state) {
            println!(
                "[Session {}] Invalid state {} for {} (cmd: {})",
                ctx.session_id, current_state, cmd_name, cmd
            );
            return Ok(PacketHandlerResult::Error(format!(
                "Invalid state {} for command {}",
                current_state, cmd
            )));
        }

        // Create reader and handle
        let mut reader = PacketReader::new(packet.data.clone());

        println!(
            "[Session {}] Handling {} (cmd: {}, {} bytes)",
            ctx.session_id,
            handler.name(),
            cmd,
            packet.data.len()
        );

        handler.handle(&mut reader, ctx).await
    }

    /// Get number of registered handlers
    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }
}

impl Default for PacketRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for packet registry
pub struct PacketRegistryBuilder {
    registry: PacketRegistry,
}

impl PacketRegistryBuilder {
    pub fn new() -> Self {
        Self {
            registry: PacketRegistry::new(),
        }
    }

    pub fn register<H: PacketHandler + 'static>(mut self, handler: H) -> Self {
        self.registry.register(handler);
        self
    }

    pub fn build(self) -> PacketRegistry {
        self.registry
    }
}

impl Default for PacketRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

