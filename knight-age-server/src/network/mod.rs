// ========== Core Modules ==========
pub mod connection;
pub mod context;
pub mod crypto;
pub mod opcode;
pub mod packet;
pub mod session;
pub mod state;
pub mod worker;

// ========== Handler Modules ==========
pub mod handler;
pub mod server_message;


// Connection
pub use connection::{ConnectionReader, ConnectionWriter, SharedWriter};

// Context & State
pub use context::PacketContext;
pub use state::ConnectionState;

// Packet
pub use packet::{reader::PacketReader, writer::PacketWriter, Packet};
pub use opcode::cmd;

// Handlers
pub use handler::{CmLogin, CmServerInfo, PacketHandler, PacketHandlerResult, PacketRegistry};

// Server Messages
pub use server_message::{
    PacketContextExt, RebuildItemData, ServerMessage,
    SmChatPopup, SmChatTab, SmDeleteRms, SmInfoEasy, SmInfoFromServer,
    SmLevelUp, SmLoginFail, SmLoginOk, SmNameServer, SmSetExp, SmUpdateData,
};

// Session & Worker
pub use session::{Session, SessionManager};
pub use worker::{ServerConfig, WorkerPool, WorkerTask};

// Crypto
pub use crypto::XorCipher;
