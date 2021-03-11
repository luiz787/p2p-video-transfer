use common::{ChunkListMessage, Message, ResponseInfo};
use core::panic;
use std::{
    collections::HashMap,
    env,
    fs::{File, OpenOptions},
    io::{ErrorKind, Write},
    net::{SocketAddr, UdpSocket},
    time::{Duration, Instant},
};

mod client_config;
use client_config::ClientConfig;

fn main() {
    let config = ClientConfig::new(env::args());
    let mut control_map = create_control_map(&config);

    let udp_socket = UdpSocket::bind(("127.0.0.1", 0)).unwrap();

    let log_file_path = get_log_file_path(&udp_socket);
    create_log_file(&log_file_path);

    // TODO: refactor to use nonblocking IO
    udp_socket
        .set_nonblocking(true)
        .expect("Failed to set socket to nonblocking mode.");

    udp_socket
        .connect(config.address)
        .expect("Falha ao conectar com o peer remoto");

    let message = ChunkListMessage::from_chunks(1, config.chunks);
    udp_socket
        .send_to(&message.serialize(), config.address)
        .expect("Falha ao enviar mensagem");

    let mut all_chunks_received = false;
    let start = Instant::now();

    // TODO: change condition to: while did not receive all chunks or not timeout
    while !all_chunks_received && !timed_out(&start) {
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
                        handle_response(data, &udp_socket, &remote_addr, &mut control_map);
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

        all_chunks_received = control_map.iter().all(|(_chunk, received)| *received);
        println!("All received? {}", all_chunks_received);
    }

    control_map
        .iter()
        .filter(|(&_chunk, &received)| !received)
        .map(|(&chunk, &_received)| format!("0.0.0.0:0 - {}\n", chunk))
        .for_each(|line| {
            write_content_to_log(&log_file_path, line);
        });
    println!("Exiting...");
}

fn create_log_file(log_file_path: &str) {
    match File::create(log_file_path) {
        Ok(_file) => {}
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {}
        Err(e) => {
            eprintln!("{}", e);
            panic!("Failed to create log file");
        }
    }
}

fn get_log_file_path(udp_socket: &UdpSocket) -> String {
    let local_ip = udp_socket
        .local_addr()
        .expect("Failed to get local address")
        .ip();

    format!("output-{}.log", local_ip)
}

fn timed_out(start: &Instant) -> bool {
    let time_elapsed = Instant::now() - *start;
    let timed_out = time_elapsed > Duration::from_secs(5);

    if timed_out {
        println!("Timed out");
    }

    timed_out
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

    if !data.chunk_list.chunks.is_empty() {
        let get_message = ChunkListMessage::from_chunks(4, data.chunk_list.chunks.clone());

        udp_socket
            .send_to(&get_message.serialize(), remote_addr)
            .expect("Falha ao enviar mensagem");
    }
}

fn handle_response(
    data: ResponseInfo,
    udp_socket: &UdpSocket,
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

    let log_file_path = get_log_file_path(&udp_socket);

    let peer_ip = remote_addr.ip();
    let peer_port = remote_addr.port();

    let content = format!("{}:{} - {}\n", peer_ip, peer_port, data.chunk_id);

    println!("{}", content);
    write_content_to_log(&log_file_path, content);
}

fn write_content_to_log(log_file_path: &str, content: String) {
    let mut log_file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(log_file_path)
        .expect("Failed to open log file");
    log_file
        .write(&content.into_bytes())
        .expect("Failed to write to log");
}
