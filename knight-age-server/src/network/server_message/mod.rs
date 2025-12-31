//! Server Message Module
//! 
//! This module provides a clean, organized way to build and send server packets.
//! Similar to Java's GlobalService pattern but adapted for Rust.
//! 
//! # Architecture
//! 
//! - `ServerMessage` trait: Base trait for all server messages
//! - `ServerService`: Helper struct for sending common packets
//! - Individual message structs: Specific packet builders
//! 
//! # Usage
//! 
//! ```rust
//! // Method 1: Using ServerMessage trait
//! let msg = SmInfoFromServer::new("Welcome!", "", 0);
//! ctx.send_message(&msg).await?;
//! 
//! // Method 2: Using ServerService
//! let service = ServerService::new(&ctx);
//! service.send_info_from_server("Welcome!").await?;
//! 
//! // Method 3: Using ctx.send_with directly
//! ctx.send_with(cmd::INFO_FROM_SERVER, |w| {
//!     w.write_string("Welcome!");
//!     w.write_string("");
//!     w.write_i8(0);
//! }).await?;
//! ```

mod traits;
mod service;
mod messages;

pub use traits::*;
pub use service::*;
pub use messages::*;

