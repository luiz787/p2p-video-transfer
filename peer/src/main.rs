use common::Message;
use std::net::UdpSocket;

fn main() {
    let udp_socket = UdpSocket::bind(("127.0.0.1", 0)).unwrap();

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
        if let Message::Hello(data) = message {
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
        }
    }
}
