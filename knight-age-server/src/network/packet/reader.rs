use std::io::{self, Cursor};
use bytes::Buf;

pub struct PacketReader {
    cursor: Cursor<Vec<u8>>,
}

impl PacketReader {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            cursor: Cursor::new(data),
        }
    }

    pub fn read_i8(&mut self) -> io::Result<i8> {
        if !self.cursor.has_remaining() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }
        Ok(self.cursor.get_i8())
    }

    pub fn read_u8(&mut self) -> io::Result<u8> {
        if !self.cursor.has_remaining() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }
        Ok(self.cursor.get_u8())
    }

    pub fn read_short(&mut self) -> io::Result<i16> {
        if self.cursor.remaining() < 2 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF Short"));
        }
        Ok(self.cursor.get_i16()) // big-endian mặc định
    }

    pub fn read_int(&mut self) -> io::Result<i32> {
        if self.cursor.remaining() < 4 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF Int"));
        }
        Ok(self.cursor.get_i32())
    }

    pub fn read_bool(&mut self) -> io::Result<bool> {
        let b = self.read_i8()?;
        Ok(b > 0)
    }

    pub fn read_string(&mut self) -> io::Result<String> {
        let len = self.read_short()?;
        if len <= 0 {
            return Ok(String::new());
        }
        let len = len as usize;
        if self.cursor.remaining() < len {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF String"));
        }
        let mut str_bytes = vec![0u8; len];
        self.cursor.copy_to_slice(&mut str_bytes);

        String::from_utf8(str_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}