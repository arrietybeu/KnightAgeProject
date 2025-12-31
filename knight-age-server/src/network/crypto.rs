/// XOR Cipher for packet encryption/decryption
/// Matches the client's readKey/writeKey implementation
#[derive(Debug, Clone)]
pub struct XorCipher {
    key: Vec<u8>,
    read_pos: usize,
    write_pos: usize,
}

impl XorCipher {
    /// Create a new XOR cipher with the given key
    pub fn new(key: Vec<u8>) -> Self {
        Self {
            key,
            read_pos: 0,
            write_pos: 0,
        }
    }

    /// Generate a random key of given length
    pub fn generate_key(length: usize) -> Vec<u8> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..length).map(|_| rng.gen::<u8>()).collect()
    }

    /// Get the raw key (before XOR transformation)
    /// This is what we send to the client
    pub fn get_raw_key(&self) -> &[u8] {
        &self.key
    }

    /// Transform key for client (client will XOR each byte with previous)
    /// Server sends: [k0, k1, k2, k3, ...]
    /// Client computes: [k0, k1^k0, k2^k1^k0, k3^k2^k1^k0, ...]
    /// So we need to send the inverse transformation
    pub fn get_key_for_client(&self) -> Vec<u8> {
        let mut client_key = self.key.clone();
        // Reverse the XOR operation that client will do
        // Client does: key[j+1] ^= key[j]
        // So we need to send values that after client's XOR gives our key
        for i in (1..client_key.len()).rev() {
            client_key[i] ^= client_key[i - 1];
        }
        client_key
    }

    /// Decrypt a single byte (for reading from client)
    pub fn decrypt_byte(&mut self, byte: u8) -> u8 {
        let result = byte ^ self.key[self.read_pos];
        self.read_pos = (self.read_pos + 1) % self.key.len();
        result
    }

    /// Encrypt a single byte (for writing to client)
    pub fn encrypt_byte(&mut self, byte: u8) -> u8 {
        let result = byte ^ self.key[self.write_pos];
        self.write_pos = (self.write_pos + 1) % self.key.len();
        result
    }

    /// Decrypt a buffer in place
    pub fn decrypt_buffer(&mut self, buffer: &mut [u8]) {
        for byte in buffer.iter_mut() {
            *byte = self.decrypt_byte(*byte);
        }
    }

    /// Encrypt a buffer in place
    pub fn encrypt_buffer(&mut self, buffer: &mut [u8]) {
        for byte in buffer.iter_mut() {
            *byte = self.encrypt_byte(*byte);
        }
    }

    /// Reset read position
    pub fn reset_read_pos(&mut self) {
        self.read_pos = 0;
    }

    /// Reset write position
    pub fn reset_write_pos(&mut self) {
        self.write_pos = 0;
    }

    /// Reset both positions
    pub fn reset(&mut self) {
        self.read_pos = 0;
        self.write_pos = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = vec![0x12, 0x34, 0x56, 0x78];
        let mut cipher1 = XorCipher::new(key.clone());
        let mut cipher2 = XorCipher::new(key);

        let original = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        let mut encrypted = original.clone();
        cipher1.encrypt_buffer(&mut encrypted);

        let mut decrypted = encrypted.clone();
        cipher2.decrypt_buffer(&mut decrypted);

        assert_eq!(original, decrypted);
    }
}

