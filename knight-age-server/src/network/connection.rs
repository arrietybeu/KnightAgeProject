use crate::network::crypto::XorCipher;
use crate::network::packet::Packet;
use std::io;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::Mutex;

/// Special commands that use 4-byte length instead of 2-byte
const SPECIAL_CMDS: [i8; 4] = [-51, -52, -54, 126];

/// Command for key exchange
pub const CMD_KEY_EXCHANGE: i8 = -40;

/// Connection reader - reads packets from client
pub struct ConnectionReader {
    reader: BufReader<OwnedReadHalf>,
    cipher: Option<XorCipher>,
    key_exchanged: bool,
}

impl ConnectionReader {
    pub fn new(read_half: OwnedReadHalf) -> Self {
        Self {
            reader: BufReader::new(read_half),
            cipher: None,
            key_exchanged: false,
        }
    }

    /// Set the cipher after key exchange
    pub fn set_cipher(&mut self, cipher: XorCipher) {
        self.cipher = Some(cipher);
        self.key_exchanged = true;
    }

    /// Check if key has been exchanged
    pub fn is_key_exchanged(&self) -> bool {
        self.key_exchanged
    }

    /// Read a single byte and decrypt if needed
    async fn read_byte(&mut self) -> io::Result<u8> {
        let byte = self.reader.read_u8().await?;
        if let Some(ref mut cipher) = self.cipher {
            Ok(cipher.decrypt_byte(byte))
        } else {
            Ok(byte)
        }
    }

    /// Read a packet from the connection
    pub async fn read_packet(&mut self) -> io::Result<Packet> {
        // Read command byte
        let cmd = self.read_byte().await? as i8;

        // Determine length size based on command
        let data_len: usize = if self.key_exchanged {
            if SPECIAL_CMDS.contains(&cmd) {
                // Special commands use 4-byte length (big-endian)
                let b0 = self.read_byte().await?;
                let b1 = self.read_byte().await?;
                let b2 = self.read_byte().await?;
                let b3 = self.read_byte().await?;
                ((b0 as usize) << 24) | ((b1 as usize) << 16) | ((b2 as usize) << 8) | (b3 as usize)
            } else {
                // Normal commands use 2-byte length (big-endian)
                let b0 = self.read_byte().await?;
                let b1 = self.read_byte().await?;
                ((b0 as usize) << 8) | (b1 as usize)
            }
        } else {
            // Before key exchange, read 2 bytes as big-endian u16
            let len_bytes = self.reader.read_u16().await?;
            len_bytes as usize
        };

        // Read data
        let mut data = vec![0u8; data_len];
        if data_len > 0 {
            self.reader.read_exact(&mut data).await?;
            // Decrypt data if cipher is set
            if let Some(ref mut cipher) = self.cipher {
                cipher.decrypt_buffer(&mut data);
            }
        }

        Ok(Packet::with_data(cmd, data))
    }
}

/// Connection writer - writes packets to client
pub struct ConnectionWriter {
    writer: BufWriter<OwnedWriteHalf>,
    cipher: Option<XorCipher>,
    key_exchanged: bool,
}

impl ConnectionWriter {
    pub fn new(write_half: OwnedWriteHalf) -> Self {
        Self {
            writer: BufWriter::new(write_half),
            cipher: None,
            key_exchanged: false,
        }
    }

    /// Set the cipher after key exchange
    pub fn set_cipher(&mut self, cipher: XorCipher) {
        self.cipher = Some(cipher);
        self.key_exchanged = true;
    }

    /// Check if key has been exchanged
    pub fn is_key_exchanged(&self) -> bool {
        self.key_exchanged
    }

    /// Write a single byte and encrypt if needed
    async fn write_byte(&mut self, byte: u8) -> io::Result<()> {
        let byte_to_write = if let Some(ref mut cipher) = self.cipher {
            cipher.encrypt_byte(byte)
        } else {
            byte
        };
        self.writer.write_u8(byte_to_write).await
    }

    /// Write a packet to the connection
    pub async fn write_packet(&mut self, packet: &Packet) -> io::Result<()> {
        let data_len = packet.data.len();

        if self.key_exchanged {
            // Encrypted write
            self.write_byte(packet.cmd as u8).await?;

            // Write length (2 bytes, big-endian)
            self.write_byte((data_len >> 8) as u8).await?;
            self.write_byte((data_len & 0xFF) as u8).await?;

            // Write data (encrypted)
            for &byte in &packet.data {
                self.write_byte(byte).await?;
            }
        } else {
            // Unencrypted write (for key exchange response)
            self.writer.write_i8(packet.cmd).await?;
            self.writer.write_u16(data_len as u16).await?;
            if data_len > 0 {
                self.writer.write_all(&packet.data).await?;
            }
        }

        self.writer.flush().await
    }

    /// Write key exchange packet (special handling - no encryption)
    pub async fn write_key_exchange(&mut self, key_data: &[u8]) -> io::Result<()> {
        // Key exchange is always unencrypted
        self.writer.write_i8(CMD_KEY_EXCHANGE).await?;
        
        // Length = 1 (key length byte) + key data
        let total_len = 1 + key_data.len();
        self.writer.write_u16(total_len as u16).await?;
        
        // Write key length
        self.writer.write_u8(key_data.len() as u8).await?;
        
        // Write key data
        self.writer.write_all(key_data).await?;
        
        self.writer.flush().await
    }
}

/// Thread-safe connection writer wrapper
pub type SharedWriter = Arc<Mutex<ConnectionWriter>>;

pub fn create_shared_writer(writer: ConnectionWriter) -> SharedWriter {
    Arc::new(Mutex::new(writer))
}

