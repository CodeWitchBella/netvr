use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DiscoveryResponse {
    header: [u8; 5],
    pub port: u16,
}

impl DiscoveryResponse {
    pub fn new(port: u16) -> Self {
        Self {
            header: [b'n', b'e', b't', b'v', b'r'],
            port,
        }
    }
    pub fn validate_header(&self) -> bool {
        self.header == [b'n', b'e', b't', b'v', b'r']
    }
}
