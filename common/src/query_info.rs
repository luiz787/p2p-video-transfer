use crate::byte_utils;
use crate::chunk_list::ChunkList;
use std::net::SocketAddr;

pub struct QueryInfo {
    pub message_type: u16,
    pub address: SocketAddr,
    pub peer_ttl: u16,
    pub chunk_info: ChunkList,
}

impl QueryInfo {
    pub fn new(message: &[u8], bytes_read: usize) -> Result<QueryInfo, &'static str> {
        if bytes_read < 12 {
            return Err(
                "Foram lidos menos de 12 bytes para uma mensagem que deve conter no mínimo 12 bytes",
            );
        }

        let message_type = byte_utils::u16_from_u8_array(&message[0..2]);
        let address = &message[2..8];
        let ip_octets = &address[0..4];
        let port = byte_utils::u16_from_u8_array(&address[4..6]);
        let address: SocketAddr = format!(
            "{}.{}.{}.{}:{}",
            ip_octets[0], ip_octets[1], ip_octets[2], ip_octets[3], port
        )
        .parse()
        .expect("Failed to parse address");

        let peer_ttl = byte_utils::u16_from_u8_array(&message[8..10]);
        let chunk_info = ChunkList::new(&message[10..], bytes_read)?;

        Ok(QueryInfo {
            message_type,
            address,
            peer_ttl,
            chunk_info,
        })
    }
}
