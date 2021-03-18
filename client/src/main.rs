use common::{ChunkListMessage, Message, ResponseInfo};
use core::panic;
use std::{
    collections::HashMap,
    env,
    io::ErrorKind,
    net::{SocketAddr, UdpSocket},
    time::{Duration, Instant},
};

mod client_config;
use client_config::ClientConfig;

mod logger;
use logger::Logger;

mod chunk_control_data;
use chunk_control_data::ChunkControlData;

fn main() {
    let config = ClientConfig::new(env::args());
    let mut control_map = create_control_map(&config);

    let udp_socket = UdpSocket::bind(("0.0.0.0", 0)).unwrap();

    let local_ip = udp_socket
        .local_addr()
        .expect("Failed to get local address")
        .ip();
    let logger = Logger::new(local_ip);

    udp_socket
        .set_nonblocking(true)
        .expect("Failed to set socket to nonblocking mode.");

    let message = ChunkListMessage::from_chunks(1, config.chunks);
    udp_socket
        .send_to(&message.serialize(), config.address)
        .expect("Falha ao enviar mensagem");

    let mut all_chunks_received = false;
    let start = Instant::now();

    while !all_chunks_received && !timed_out(&start) {
        let mut buffer = [0; 60 * 1024];

        let result = udp_socket.recv_from(&mut buffer);
        match result {
            Ok((bytes_read, remote_addr)) => {
                println!("Received {} bytes", bytes_read);
                let message = Message::new(&buffer, bytes_read).expect("Failed to parse message");

                match message {
                    Message::ChunkInfo(data) => {
                        handle_chunk_info(&udp_socket, &data, &remote_addr, &mut control_map);
                    }
                    Message::Response(data) => {
                        handle_response(data, &logger, &remote_addr, &mut control_map);
                    }
                    _ => {}
                }
            }
            Err(ref err) if err.kind() != ErrorKind::WouldBlock => {
                panic!("Failed to read from udp socket");
            }
            _ => {
                continue;
            }
        }

        all_chunks_received = control_map
            .iter()
            .all(|(_chunk, chunk_control_data)| chunk_control_data.received);
        println!("All received? {}", all_chunks_received);
    }

    control_map
        .iter()
        .filter(|(&_chunk, &chunk_control_data)| !chunk_control_data.received)
        .map(|(&chunk, &_chunk_control_data)| format!("0.0.0.0:0 - {}\n", chunk))
        .for_each(|line| {
            logger.log(line);
        });
    println!("Exiting...");
}

fn timed_out(start: &Instant) -> bool {
    let time_elapsed = Instant::now() - *start;
    let timed_out = time_elapsed > Duration::from_secs(5);

    if timed_out {
        println!("Timed out");
    }

    timed_out
}

fn create_control_map(config: &ClientConfig) -> HashMap<u16, ChunkControlData> {
    config
        .chunks
        .iter()
        .map(|chunk| {
            (
                *chunk,
                ChunkControlData {
                    received: false,
                    sent_get: false,
                },
            )
        })
        .collect()
}

fn handle_chunk_info(
    udp_socket: &UdpSocket,
    data: &ChunkListMessage,
    remote_addr: &SocketAddr,
    control_map: &mut HashMap<u16, ChunkControlData>,
) {
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

    if !data.chunk_list.chunks.is_empty() {
        let mut needed_chunks = Vec::new();

        for chunk in &data.chunk_list.chunks {
            let value = control_map.get(chunk);
            let should_include_chunk_in_get_message = match value {
                Some(chunk_control_data) => !chunk_control_data.sent_get,
                None => true,
            };

            if should_include_chunk_in_get_message {
                needed_chunks.push(*chunk);
            }
        }

        if !needed_chunks.is_empty() {
            let get_message = ChunkListMessage::from_chunks(4, needed_chunks.clone());

            udp_socket
                .send_to(&get_message.serialize(), remote_addr)
                .expect("Falha ao enviar mensagem");

            for chunk in &needed_chunks {
                control_map.get_mut(chunk).expect("Unknown error").sent_get = true;
            }
        }
    }
}

fn handle_response(
    data: ResponseInfo,
    logger: &Logger,
    remote_addr: &SocketAddr,
    control_map: &mut HashMap<u16, ChunkControlData>,
) {
    println!(
        "Received chunk {} from peer {}.",
        data.chunk_id, remote_addr
    );

    control_map
        .get_mut(&data.chunk_id)
        .expect("Received unwanted chunk")
        .received = true;

    let peer_ip = remote_addr.ip();
    let peer_port = remote_addr.port();

    let content = format!("{}:{} - {}\n", peer_ip, peer_port, data.chunk_id);

    println!("{}", content);
    logger.log(content);
}
