use std::{collections::HashMap, fs};

use crate::peer_config::PeerConfig;

pub type ChunkId = u16;
pub type Chunk = Vec<u8>;
pub struct ChunkManager {
    map: HashMap<ChunkId, Chunk>,
}

impl ChunkManager {
    pub fn new(config: &PeerConfig) -> ChunkManager {
        let kv_file_contents =
            fs::read_to_string(&config.kv_file_path).expect("Unable to open key-value file");

        let mut map: HashMap<ChunkId, Chunk> = HashMap::new();
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

        ChunkManager { map }
    }

    pub fn contains(&self, key: &ChunkId) -> bool {
        self.map.contains_key(key)
    }

    pub fn get(&self, key: &ChunkId) -> Option<&Chunk> {
        self.map.get(key)
    }
}
