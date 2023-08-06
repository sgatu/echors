use std::sync::Arc;

use parking_lot::RwLock;
use std::string::String;

use crate::state::serverstate::ServerState;

pub struct InfoCmd {}
impl InfoCmd {
    pub fn execute(server_state_rwl: &Arc<RwLock<ServerState>>) -> Result<Option<Vec<u8>>, String> {
        let state = server_state_rwl.read();
        return Ok(Some(state.to_string().as_bytes().to_vec()));
    }
}
