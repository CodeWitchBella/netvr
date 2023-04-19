use std::collections::HashMap;

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

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct LocalStateSnapshot {
    pub controllers: Vec<Pose>,
    pub views: Vec<Pose>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct RemoteStateSnapshot {
    /// Makes sure that we do not apply older snapshots, if they arrive out of
    /// order.
    pub order: usize,
    pub clients: HashMap<usize, LocalStateSnapshot>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Heartbeat {
    _buf: [u8; 5],
}
impl Default for Heartbeat {
    fn default() -> Self {
        Self {
            _buf: [b'h', b'e', b'l', b'l', b'o'],
        }
    }
}
