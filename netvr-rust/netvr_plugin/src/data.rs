#[derive(Clone)]
pub(crate) struct Data {
    pub(crate) state: netvr_data::net::LocalStateSnapshot,
}
impl Data {
    pub(crate) fn new() -> Data {
        Data {
            state: netvr_data::net::LocalStateSnapshot::default(),
        }
    }
}
