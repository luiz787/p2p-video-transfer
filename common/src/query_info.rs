use crate::byte_utils;
use crate::chunk_list::ChunkList;
use core::panic;
use std::net::{IpAddr, SocketAddr};

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
                "Less than 12 bytes read for message that should contain at least 12 bytes.",
            );
        }

        let message_type = byte_utils::u16_from_u8_array(&message[0..2]);

        println!("[DEBUG] MessageType={}", message_type);

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

    pub fn from_chunks(address: SocketAddr, chunk_info: ChunkList) -> QueryInfo {
        QueryInfo {
            message_type: 2,
            address,
            peer_ttl: 3,
            chunk_info,
        }
    }

    pub fn with_decremented_ttl(&self) -> QueryInfo {
        QueryInfo {
            message_type: self.message_type,
            address: self.address,
            chunk_info: self.chunk_info.clone(),
            peer_ttl: self.peer_ttl - 1,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.extend(self.message_type.to_be_bytes().iter());

        let ip_octets = match self.address.ip() {
            IpAddr::V4(ip) => ip.octets(),
            _ => panic!("IPv6 not supported"),
        };

        data.append(&mut ip_octets.to_vec());
        data.extend(self.address.port().to_be_bytes().iter());
        data.extend(self.peer_ttl.to_be_bytes().iter());
        data.append(&mut self.chunk_info.serialize());

        data
    }
}
