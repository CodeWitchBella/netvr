use crate::client::Client;

pub(crate) struct Server {
    pub(crate) clients: Vec<Client>,
}

impl Server {
    pub fn new() -> Self {
        Self { clients: vec![] }
    }
}
