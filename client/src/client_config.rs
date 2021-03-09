use std::{env, net::SocketAddr};

#[derive(Debug)]
pub struct ClientConfig {
    pub address: SocketAddr,
    pub chunks: Vec<u16>,
}

impl ClientConfig {
    pub fn new(mut args: env::Args) -> ClientConfig {
        args.next();

        let address = args.next().expect("Peer address not specified");

        let address: SocketAddr = address.parse().expect("Failed to parse peer address");

        let chunks = args.next().expect("Chunk numbers not specified");

        let chunks = chunks
            .split(",")
            .map(|chunk| chunk.parse::<u16>().expect("Failed to parse chunk numbers"))
            .collect();

        return ClientConfig { address, chunks };
    }
}
