use std::env;

mod client_config;
use client_config::ClientConfig;

fn main() {
    let config = ClientConfig::new(env::args());

    println!("{:?}", config);
}
