use common::{ChunkListMessage, Message, QueryInfo, ResponseInfo};
use std::{
    env,
    net::{SocketAddr, UdpSocket},
};

mod peer_config;
use peer_config::PeerConfig;

mod chunk_manager;
use chunk_manager::ChunkManager;

fn main() {
    let config = PeerConfig::new(env::args());
    let chunk_manager = ChunkManager::new(&config);
    let udp_socket = UdpSocket::bind((config.ip, config.port)).unwrap();

    println!("UDP bound to {}", udp_socket.local_addr().unwrap().port());

    loop {
        let mut buffer = [0; 60 * 1024];
        let (bytes_read, remote_address) = udp_socket
            .recv_from(&mut buffer)
            .expect("Failed to read from udp socket");

        println!("Read {} bytes from {}", bytes_read, remote_address);

        let message = Message::new(&buffer, bytes_read).expect("Failed to parse message");
        match message {
            Message::Hello(data) => {
                handle_hello(&chunk_manager, &udp_socket, data, &remote_address, &config);
            }
            Message::Get(data) => {
                handle_get(&chunk_manager, &udp_socket, data, &remote_address);
            }
            Message::Query(query_info) => {
                handle_query(
                    &chunk_manager,
                    &udp_socket,
                    query_info,
                    &config,
                    &remote_address,
                );
            }
            _ => {}
        }
    }
}

fn handle_hello(
    chunk_manager: &ChunkManager,
    udp_socket: &UdpSocket,
    data: ChunkListMessage,
    remote_address: &SocketAddr,
    config: &PeerConfig,
) {
    println!(
        "Got hello message! Client is asking for {} chunks",
        data.chunk_list.chunks.len()
    );

    println!(
        "{}",
        data.chunk_list
            .chunks
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );

    let mut available_chunks = Vec::new();
    print!("Available chunks: ");
    for chunk in &data.chunk_list.chunks {
        if chunk_manager.contains(chunk) {
            print!("{}", chunk);
            available_chunks.push(*chunk);
        }
    }
    println!();

    if !available_chunks.is_empty() {
        let message = ChunkListMessage::from_chunks(3, available_chunks);
        let amt = udp_socket
            .send_to(&message.serialize(), remote_address)
            .expect("Failed to communicate with client");
        println!("Sent {} bytes", amt);
    }

    let message = QueryInfo::from_chunks(*remote_address, data.chunk_list.clone());

    println!("Sending query message");

    println!(
        "{}",
        data.chunk_list
            .chunks
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );

    for peer in &config.known_peers {
        let amt = udp_socket
            .send_to(&message.serialize(), peer)
            .expect("Failed to communicate with client");
        println!("Sent {} bytes", amt);
    }
}

fn handle_get(
    chunk_manager: &ChunkManager,
    udp_socket: &UdpSocket,
    data: ChunkListMessage,
    remote_address: &SocketAddr,
) {
    println!(
        "Got GET message! Client is asking for {} chunks",
        data.chunk_list.chunks.len()
    );

    println!(
        "{}",
        data.chunk_list
            .chunks
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );

    for chunk_id in data.chunk_list.chunks {
        let chunk = chunk_manager.get(&chunk_id);

        if let Some(chunk_data) = chunk {
            println!("Sending chunk {} to client {}", chunk_id, remote_address);
            let mut response_message = ResponseInfo::from_chunk(chunk_id, chunk_data.clone());
            udp_socket
                .send_to(&mut response_message.serialize(), remote_address)
                .expect("Failed to communicate with client");
        }
    }
}

fn handle_query(
    chunk_manager: &ChunkManager,
    udp_socket: &UdpSocket,
    data: QueryInfo,
    config: &PeerConfig,
    remote_address: &SocketAddr,
) {
    println!(
        "Got QUERY message! Client is asking for chunks: {}",
        data.chunk_info
            .chunks
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );

    let available_chunks: Vec<u16> = data
        .chunk_info
        .chunks
        .iter()
        .filter(|&chunk| chunk_manager.contains(chunk))
        .map(|&chunk| chunk)
        .collect();

    if !available_chunks.is_empty() {
        let message = ChunkListMessage::from_chunks(3, available_chunks);
        let amt = udp_socket
            .send_to(&message.serialize(), data.address)
            .expect("Failed to communicate with client");
        println!("Sent {} bytes to {}", amt, data.address);
    }

    let message = data.with_decremented_ttl();
    if message.peer_ttl > 0 {
        println!("Sending query message");

        println!(
            "{}",
            message
                .chunk_info
                .chunks
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );

        config
            .known_peers
            .iter()
            .filter(|&peer| peer != remote_address)
            .for_each(|peer| {
                let amt = udp_socket
                    .send_to(&message.serialize(), peer)
                    .expect("Failed to communicate with client");
                println!("Sent {} bytes", amt);
            });
    }
}
