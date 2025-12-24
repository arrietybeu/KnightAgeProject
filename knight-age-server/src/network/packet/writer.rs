use bytes::{BytesMut, BufMut};

pub struct PacketWriter {
    buffer: BytesMut,
}

impl PacketWriter {
    pub fn new() -> Self {
        Self {
            buffer: BytesMut::with_capacity(1024),
        }
    }

    pub fn write_i8(&mut self, v: i8) {
        self.buffer.put_i8(v);
    }

    pub fn write_u8(&mut self, v: u8) {
        self.buffer.put_u8(v);
    }

    pub fn write_short(&mut self, v: i16) {
        self.buffer.put_i16(v);
    }

    pub fn write_int(&mut self, v: i32) {
        self.buffer.put_i32(v);
    }

    pub fn write_bool(&mut self, v: bool) {
        self.buffer.put_i8(if v { 1 } else { 0 });
    }

    pub fn write_string(&mut self, v: &str) {
        let bytes = v.as_bytes();
        self.write_short(bytes.len() as i16);
        self.buffer.put_slice(bytes);
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.buffer.to_vec()
    }
}