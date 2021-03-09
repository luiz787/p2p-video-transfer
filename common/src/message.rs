use crate::chunk_list::ChunkListMessage;
use crate::query_info::QueryInfo;
use crate::response_info::ResponseInfo;

pub enum Message {
    Hello(ChunkListMessage),
    Get(ChunkListMessage),
    Query(QueryInfo),
    ChunkInfo(ChunkListMessage),
    Response(ResponseInfo),
}

impl Message {
    pub fn new(message: &[u8], bytes_read: usize) -> Result<Message, &'static str> {
        if bytes_read < 2 {
            return Err("Less than 2 bytes read");
        }

        let message_type = message[1];
        match message_type {
            1 => Ok(Self::Hello(ChunkListMessage::new(message, bytes_read)?)),
            2 => Ok(Self::Query(QueryInfo::new(message, bytes_read)?)),
            3 => Ok(Self::ChunkInfo(ChunkListMessage::new(message, bytes_read)?)),
            4 => Ok(Self::Get(ChunkListMessage::new(message, bytes_read)?)),
            5 => Ok(Self::Response(ResponseInfo::new(message, bytes_read)?)),
            _ => Err("Unknown message type"),
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        match self {
            Message::Hello(list) | Message::ChunkInfo(list) | Message::Get(list) => {
                list.serialize()
            }
            Message::Query(_query_info) => todo!("Implementar serialização"),
            Message::Response(_response_info) => todo!("Implementar serialização"),
        }
    }
}
