use crate::byte_utils;

pub struct ChunkListMessage {
    pub message_type: u16,
    pub chunk_list: ChunkList,
}

impl ChunkListMessage {
    pub fn new(message: &[u8], bytes_read: usize) -> Result<ChunkListMessage, &'static str> {
        if bytes_read < 4 {
            return Err(
                "Less than 4 bytes read for message that should contain at least 4 bytes.",
            );
        }

        let message_type = byte_utils::u16_from_u8_array(&message[0..2]);
        let chunk_list = ChunkList::new(&message[2..], bytes_read - 2)?;
        Ok(ChunkListMessage {
            message_type,
            chunk_list,
        })
    }

    pub fn from_chunks(message_type: u16, chunks: Vec<u16>) -> ChunkListMessage {
        ChunkListMessage {
            message_type,
            chunk_list: ChunkList {
                amount_of_chunks: chunks.len() as u16,
                chunks,
            },
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.extend(self.message_type.to_be_bytes().iter());
        data.append(&mut self.chunk_list.serialize());

        data
    }
}

#[derive(Clone)]
pub struct ChunkList {
    pub amount_of_chunks: u16,
    pub chunks: Vec<u16>,
}

impl ChunkList {
    pub fn new(message: &[u8], bytes_read: usize) -> Result<ChunkList, &'static str> {
        if bytes_read < 2 {
            return Err(
                "Foram lidos menos de 2 bytes para uma mensagem que deve conter no mínimo 2 bytes",
            );
        }

        let amount_of_chunks = byte_utils::u16_from_u8_array(&message[0..2]);
        let slice_end = (amount_of_chunks * 2 + 2) as usize;
        let raw_bytes = &message[2..slice_end];

        let mut chunks = Vec::new();

        for i in (0..raw_bytes.len()).step_by(2) {
            let val = &raw_bytes[i..i + 2];
            let val = byte_utils::u16_from_u8_array(val);
            chunks.push(val);
        }

        Ok(ChunkList {
            amount_of_chunks,
            chunks,
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();

        data.extend(self.amount_of_chunks.to_be_bytes().iter());
        let chunks = &self.chunks;
        let mut chunks = chunks
            .iter()
            .flat_map(|chunk| Vec::from(chunk.to_be_bytes()))
            .collect::<Vec<_>>();

        data.append(&mut chunks);
        data
    }
}
