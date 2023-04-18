#[derive(Clone)]
pub(crate) struct Data {
    pub(crate) state: netvr_data::net::LocalState,
}
impl Data {
    pub(crate) fn new() -> Data {
        Data {
            state: netvr_data::net::LocalState::default(),
        }
    }
}
