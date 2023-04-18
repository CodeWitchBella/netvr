use serde::{Deserialize, Serialize};

use crate::Pose;

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

#[derive(Serialize, Deserialize, Debug)]
pub enum ConfigurationUp {
    Hello,
}

#[derive(Serialize, Deserialize)]
pub struct ConfigurationDown {
    header: [u8; 5],
    pub port: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DatagramUp {
    pub state: LocalState,
}

#[derive(Serialize, Deserialize)]
pub struct DatagramDown {
    header: [u8; 5],
    pub port: u16,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct LocalState {
    pub controllers: Vec<Pose>,
    pub views: Vec<Pose>,
}
