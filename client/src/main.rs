use common::{ChunkListMessage, Message, ResponseInfo};
use core::panic;
use std::{
    collections::HashMap,
    env,
    io::ErrorKind,
    net::{SocketAddr, UdpSocket},
};

mod client_config;
use client_config::ClientConfig;

fn main() {
    let config = ClientConfig::new(env::args());
    let mut control_map = create_control_map(&config);

    let message = ChunkListMessage::from_chunks(1, config.chunks);

    let udp_socket = UdpSocket::bind(("127.0.0.1", 0)).unwrap();

    // TODO: refactor to use nonblocking IO
    /*udp_socket
    .set_nonblocking(true)
    .expect("Failed to set socket to nonblocking mode.");*/

    udp_socket
        .connect(config.address)
        .expect("Falha ao conectar com o peer remoto");
    udp_socket
        .send_to(&message.serialize(), config.address)
        .expect("Falha ao enviar mensagem");

    let mut all_chunks_received = false;

    // TODO: change condition to: while did not receive all chunks or not timeout
    while !all_chunks_received {
        println!("Missing chunks:");

        for (c, received) in control_map.iter() {
            if !received {
                println!("{}", c);
            }
        }

        let mut buffer = [0; 60 * 1024];
        let result = udp_socket.recv_from(&mut buffer);
        match result {
            Ok((bytes_read, remote_addr)) => {
                println!("Received {} bytes", bytes_read);
                let message = Message::new(&buffer, bytes_read).expect("Failed to parse message");

                match message {
                    Message::ChunkInfo(data) => {
                        handle_chunk_info(&udp_socket, &data, &remote_addr);
                    }
                    Message::Response(data) => {
                        handle_response(data, &remote_addr, &mut control_map);
                    }
                    _ => {}
                }
            }
            Err(ref err) if err.kind() != ErrorKind::WouldBlock => {
                panic!("Failed to read from udp socket");
            }
            _ => {}
        }

        all_chunks_received = control_map.iter().all(|(_chunk, received)| *received);
        println!("All received? {}", all_chunks_received);
    }
}

fn create_control_map(config: &ClientConfig) -> HashMap<u16, bool> {
    config.chunks.iter().map(|chunk| (*chunk, false)).collect()
}

fn handle_chunk_info(udp_socket: &UdpSocket, data: &ChunkListMessage, remote_addr: &SocketAddr) {
    println!(
        "Got ChunkInfo message! Peer {} has {} chunks",
        remote_addr,
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

    let get_message = ChunkListMessage::from_chunks(4, data.chunk_list.chunks.clone());

    udp_socket
        .send_to(&get_message.serialize(), remote_addr)
        .expect("Falha ao enviar mensagem");
}

fn handle_response(
    data: ResponseInfo,
    remote_addr: &SocketAddr,
    control_map: &mut HashMap<u16, bool>,
) {
    println!(
        "Received chunk {} from peer {}.",
        data.chunk_id, remote_addr
    );

    *control_map
        .get_mut(&data.chunk_id)
        .expect("Received unwanted chunk") = true;
}
