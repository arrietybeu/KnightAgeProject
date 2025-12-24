pub mod reader;
pub mod writer;

#[derive(Debug, Clone)]
pub struct Packet {
    pub cmd: i8,
    pub data: Vec<u8>,
}

impl Packet {
    pub fn new(cmd: i8) -> Self {
        Self {
            cmd,
            data: Vec::new(),
        }
    }

    pub fn with_data(cmd: i8, data: Vec<u8>) -> Self {
        Self {
            cmd,
            data,
        }
    }
}