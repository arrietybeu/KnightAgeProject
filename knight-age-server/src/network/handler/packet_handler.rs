use crate::network::context::PacketContext;
use crate::network::packet::reader::PacketReader;
use crate::network::state::ConnectionState;
use async_trait::async_trait;
use std::io;

/// Result of packet handling
pub enum PacketHandlerResult {
    /// Packet handled successfully
    Ok,
    /// Packet handled, close connection after
    CloseConnection,
    /// Error occurred
    Error(String),
}

/// Trait for all packet handlers
/// Similar to your Java NroClientPacket abstract class
#[async_trait]
pub trait PacketHandler: Send + Sync {
    /// Get the command ID this handler processes
    fn command(&self) -> i8;

    /// Get valid states for this handler
    fn valid_states(&self) -> Vec<ConnectionState>;

    /// Check if current state is valid for this handler
    fn is_valid_state(&self, state: ConnectionState) -> bool {
        self.valid_states().contains(&state)
    }

    /// Process the packet
    /// This is called after state validation
    async fn handle(
        &self,
        reader: &mut PacketReader,
        ctx: &PacketContext,
    ) -> io::Result<PacketHandlerResult>;

    /// Get handler name for logging
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// Macro to simplify packet handler creation
#[macro_export]
macro_rules! define_packet_handler {
    (
        name: $name:ident,
        command: $cmd:expr,
        states: [$($state:expr),+ $(,)?],
        handler: |$reader:ident, $ctx:ident| $body:expr
    ) => {
        pub struct $name;

        #[async_trait::async_trait]
        impl $crate::network::handler::PacketHandler for $name {
            fn command(&self) -> i8 {
                $cmd
            }

            fn valid_states(&self) -> Vec<$crate::network::state::ConnectionState> {
                vec![$($state),+]
            }

            async fn handle(
                &self,
                $reader: &mut $crate::network::packet::reader::PacketReader,
                $ctx: &$crate::network::context::PacketContext,
            ) -> std::io::Result<$crate::network::handler::PacketHandlerResult> {
                $body
            }

            fn name(&self) -> &'static str {
                stringify!($name)
            }
        }
    };
}
