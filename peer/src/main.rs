use common::{ChunkListMessage, Message, QueryInfo, ResponseInfo};
use std::{
    collections::HashMap,
    env, fs,
    net::{SocketAddr, UdpSocket},
};

mod peer_config;
use peer_config::PeerConfig;

fn main() {
    let config = PeerConfig::new(env::args());
    let chunks_map = process_chunks(&config);
    let udp_socket = UdpSocket::bind(("0.0.0.0", config.port)).unwrap();

    println!("UDP bound to {}", udp_socket.local_addr().unwrap().port());

    loop {
        let mut buffer = [0; 1024];
        let (bytes_read, remote_address) = udp_socket
            .recv_from(&mut buffer)
            .expect("Failed to read from udp socket");

        println!("Read {} bytes from {}", bytes_read, remote_address);

        for byte in &buffer {
            print!("{}", byte);
        }
        println!();

        let message = Message::new(&buffer, bytes_read).expect("Failed to parse message");

        // TODO: use match
        match message {
            Message::Hello(data) => {
                handle_hello(&chunks_map, &udp_socket, data, &remote_address, &config);
            }
            Message::Get(data) => {
                handle_get(&chunks_map, &udp_socket, data, &remote_address);
            }
            Message::Query(query_info) => {
                handle_query(
                    &chunks_map,
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

fn process_chunks(config: &PeerConfig) -> HashMap<u16, Vec<u8>> {
    let path = env::current_dir().expect("Unable to get current directory");
    println!("The current directory is {}", path.display());

    let kv_file_contents =
        fs::read_to_string(&config.kv_file_path).expect("Unable to open key-value file");

    let mut map: HashMap<u16, Vec<u8>> = HashMap::new();
    for line in kv_file_contents.lines() {
        let mut split = line.split(": ");
        let key = split
            .next()
            .expect("Key-value file line has unknown format.")
            .parse()
            .expect("Key is not a number");
        let path = split
            .next()
            .expect("Key-value file line has unknown format.")
            .to_string();

        println!("Path: {}", path);

        let content = fs::read(path).expect("Unable to read chunk file");
        println!("Read {} bytes", content.len());

        map.insert(key, content);
    }

    map
}

fn handle_hello(
    chunks_map: &HashMap<u16, Vec<u8>>,
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
        if chunks_map.contains_key(chunk) {
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
    chunks_map: &HashMap<u16, Vec<u8>>,
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
        // TODO: improve error handling
        let chunk = chunks_map
            .get(&chunk_id)
            .expect("Client asked for chunk that is not available on this peer");

        println!("Sending chunk {} to client {}", chunk_id, remote_address);
        let mut response_message = ResponseInfo::from_chunk(chunk_id, chunk.clone());
        udp_socket
            .send_to(&mut response_message.serialize(), remote_address)
            .expect("Failed to communicate with client");
    }
}

fn handle_query(
    chunks_map: &HashMap<u16, Vec<u8>>,
    udp_socket: &UdpSocket,
    data: QueryInfo,
    config: &PeerConfig,
    remote_address: &SocketAddr,
) {
    println!(
        "Got QUERY message! Client is asking for {} chunks",
        data.chunk_info.chunks.len()
    );

    println!(
        "{}",
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
        .filter(|&chunk| chunks_map.contains_key(chunk))
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
