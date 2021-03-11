use crate::byte_utils;

pub struct ResponseInfo {
    pub message_type: u16,
    pub chunk_id: u16,
    pub chunk_size: u16,
    pub chunk: Vec<u8>,
}

impl ResponseInfo {
    pub fn new(message: &[u8], bytes_read: usize) -> Result<ResponseInfo, &'static str> {
        if bytes_read < 6 {
            return Err(
                "Foram lidos menos de 6 bytes para uma mensagem que deve conter no mínimo 6 bytes",
            );
        }

        let message_type = byte_utils::u16_from_u8_array(&message[0..2]);
        let chunk_id = byte_utils::u16_from_u8_array(&message[2..4]);
        let chunk_size = byte_utils::u16_from_u8_array(&message[4..6]);
        let chunk = Vec::from(&message[6..]);

        Ok(ResponseInfo {
            message_type,
            chunk_id,
            chunk_size,
            chunk,
        })
    }

    pub fn from_chunk(chunk_id: u16, chunk: Vec<u8>) -> ResponseInfo {
        ResponseInfo {
            message_type: 5,
            chunk_id,
            chunk_size: chunk.len() as u16,
            chunk,
        }
    }

    pub fn serialize(&mut self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.extend(self.message_type.to_be_bytes().iter());
        data.extend(self.chunk_id.to_be_bytes().iter());
        data.extend(self.chunk_size.to_be_bytes().iter());
        data.append(&mut self.chunk);

        data
    }
}
