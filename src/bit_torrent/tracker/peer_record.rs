use super::utils::u8_to_hexa;
use serde::{ser::SerializeStruct, Serialize, Serializer};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct PeerRecord {
    pub ip: String,
    pub peer_id: [u8; 20],
    pub port: u16,
    pub uploaded: u64,
    pub left: u64,
    pub event: String,
    pub created_at: Instant,
    pub updated_at: Instant,
}

impl Serialize for PeerRecord {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Record", 6)?;
        state.serialize_field("peer_id", &u8_to_hexa(&self.peer_id))?;
        state.serialize_field("ip", &self.ip)?;
        state.serialize_field("port", &self.port)?;
        state.serialize_field("event", &self.event)?;
        state.serialize_field("uploaded", &self.uploaded)?;
        state.serialize_field("left", &self.left)?;
        state.serialize_field("created_at", &self.created_at.elapsed().as_secs())?;
        state.end()
    }
}

impl PeerRecord {
    pub fn new(
        ip: String,
        peer_id: [u8; 20],
        port: u16,
        uploaded: u64,
        left: u64,
        event: String,
    ) -> Self {
        Self {
            created_at: Instant::now(),
            updated_at: Instant::now(),
            ip,
            peer_id,
            port,
            uploaded,
            left,
            event,
        }
    }

    pub fn update(&mut self, new_data: PeerRecord) {
        self.port = new_data.port;
        self.uploaded = new_data.uploaded;
        self.left = new_data.left;
        self.event = new_data.event;
        self.updated_at = Instant::now();
    }
}
