#[derive(Debug, Clone, Copy)]
pub struct ChunkControlData {
    pub received: bool,
    pub sent_get: bool,
}
