use bytes::{BytesMut, BufMut};

/// PacketWriter - builds packet data for sending to client
/// All multi-byte values are written in big-endian (network byte order)
/// Strings are written in Java modified UTF-8 format (2-byte length prefix)
pub struct PacketWriter {
    buffer: BytesMut,
}

impl PacketWriter {
    /// Create a new packet writer with default capacity
    pub fn new() -> Self {
        Self {
            buffer: BytesMut::with_capacity(1024),
        }
    }

    /// Create a new packet writer with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: BytesMut::with_capacity(capacity),
        }
    }

    /// Get current buffer length
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    // ========== Primitive Write Methods ==========

    /// Write signed byte (i8)
    pub fn write_i8(&mut self, v: i8) -> &mut Self {
        self.buffer.put_i8(v);
        self
    }

    /// Write unsigned byte (u8)
    pub fn write_u8(&mut self, v: u8) -> &mut Self {
        self.buffer.put_u8(v);
        self
    }

    /// Alias for write_i8
    pub fn write_byte(&mut self, v: i8) -> &mut Self {
        self.write_i8(v)
    }

    /// Write signed short (i16, big-endian)
    pub fn write_short(&mut self, v: i16) -> &mut Self {
        self.buffer.put_i16(v);
        self
    }

    /// Write unsigned short (u16, big-endian)
    pub fn write_ushort(&mut self, v: u16) -> &mut Self {
        self.buffer.put_u16(v);
        self
    }

    /// Write signed int (i32, big-endian)
    pub fn write_int(&mut self, v: i32) -> &mut Self {
        self.buffer.put_i32(v);
        self
    }

    /// Write unsigned int (u32, big-endian)
    pub fn write_uint(&mut self, v: u32) -> &mut Self {
        self.buffer.put_u32(v);
        self
    }

    /// Write signed long (i64, big-endian)
    pub fn write_long(&mut self, v: i64) -> &mut Self {
        self.buffer.put_i64(v);
        self
    }

    /// Write boolean (1 byte: 1 for true, 0 for false)
    pub fn write_bool(&mut self, v: bool) -> &mut Self {
        self.buffer.put_i8(if v { 1 } else { 0 });
        self
    }

    // ========== String Write Methods ==========

    /// Write string in Java modified UTF-8 format
    /// Format: [2 bytes length][string bytes]
    pub fn write_string(&mut self, v: &str) -> &mut Self {
        let bytes = v.as_bytes();
        self.write_short(bytes.len() as i16);
        self.buffer.put_slice(bytes);
        self
    }

    /// Alias for write_string (matches Java naming)
    pub fn write_utf(&mut self, v: &str) -> &mut Self {
        self.write_string(v)
    }

    // ========== Byte Array Write Methods ==========

    /// Write raw bytes
    pub fn write_bytes(&mut self, bytes: &[u8]) -> &mut Self {
        self.buffer.put_slice(bytes);
        self
    }

    /// Write signed byte array
    pub fn write_sbytes(&mut self, bytes: &[i8]) -> &mut Self {
        for &b in bytes {
            self.buffer.put_i8(b);
        }
        self
    }

    /// Write bytes with length prefix (short)
    pub fn write_bytes_with_length(&mut self, bytes: &[u8]) -> &mut Self {
        self.write_short(bytes.len() as i16);
        self.buffer.put_slice(bytes);
        self
    }

    // ========== Array Write Methods ==========

    /// Write array of strings with count prefix (u8)
    pub fn write_string_array_u8(&mut self, arr: &[&str]) -> &mut Self {
        self.write_u8(arr.len() as u8);
        for s in arr {
            self.write_string(s);
        }
        self
    }

    /// Write array of strings with count prefix (i8)
    pub fn write_string_array(&mut self, arr: &[&str]) -> &mut Self {
        self.write_i8(arr.len() as i8);
        for s in arr {
            self.write_string(s);
        }
        self
    }

    /// Write array of i16 with count prefix (i8)
    pub fn write_short_array(&mut self, arr: &[i16]) -> &mut Self {
        self.write_i8(arr.len() as i8);
        for &v in arr {
            self.write_short(v);
        }
        self
    }

    // ========== Consume Methods ==========

    /// Consume writer and return buffer as Vec<u8>
    pub fn into_inner(self) -> Vec<u8> {
        self.buffer.to_vec()
    }

    /// Consume writer and return buffer as BytesMut
    pub fn into_bytes(self) -> BytesMut {
        self.buffer
    }

    /// Get reference to internal buffer
    pub fn as_bytes(&self) -> &[u8] {
        &self.buffer
    }
}

impl Default for PacketWriter {
    fn default() -> Self {
        Self::new()
    }
}

// Allow chaining from the start
impl From<PacketWriter> for Vec<u8> {
    fn from(writer: PacketWriter) -> Self {
        writer.into_inner()
    }
}