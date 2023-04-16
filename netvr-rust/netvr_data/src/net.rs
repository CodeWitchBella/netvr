use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DiscoveryResponse {
    header: [u8; 5],
}

impl DiscoveryResponse {
    pub fn validate_header(&self) -> bool {
        self.header == [b'n', b'e', b't', b'v', b'r']
    }
}

impl Default for DiscoveryResponse {
    fn default() -> Self {
        Self {
            header: [b'n', b'e', b't', b'v', b'r'],
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum ConfigurationUp {
    Hello,
}

#[derive(Serialize, Deserialize)]
pub struct ConfigurationDown {
    header: [u8; 5],
    pub port: u16,
}

#[derive(Serialize, Deserialize)]
pub struct DatagramUp {
    header: [u8; 5],
    pub port: u16,
}

#[derive(Serialize, Deserialize)]
pub struct DatagramDown {
    header: [u8; 5],
    pub port: u16,
}
