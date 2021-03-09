use common::ChunkListMessage;
use std::{env, net::UdpSocket};

mod client_config;
use client_config::ClientConfig;

fn main() {
    let config = ClientConfig::new(env::args());

    let message = ChunkListMessage::from_chunks(1, config.chunks);

    let udp = UdpSocket::bind(("127.0.0.1", 0)).unwrap();
    udp.connect(config.address)
        .expect("Falha ao conectar com o peer remoto");
    udp.send_to(&message.serialize(), config.address)
        .expect("Falha ao enviar mensagem");
}
