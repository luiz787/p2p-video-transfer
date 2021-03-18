use std::{env, net::SocketAddr};

#[derive(Debug)]
pub struct PeerConfig {
    pub port: u16,
    pub kv_file_path: String,
    pub known_peers: Vec<SocketAddr>,
}

impl PeerConfig {
    pub fn new(mut args: env::Args) -> PeerConfig {
        args.next();

        let port = args
            .next()
            .expect("Port not specified")
            .parse()
            .expect("Unable to parse port to 2 byte integer");

        let kv_file_path = args.next().expect("Key-values file path not specified");

        let mut known_peers = Vec::new();
        while let Some(addr) = args.next() {
            let peer_address: SocketAddr = addr.parse().expect("Failed to parse address");

            known_peers.push(peer_address);
        }

        return PeerConfig {
            port,
            kv_file_path,
            known_peers,
        };
    }
}
