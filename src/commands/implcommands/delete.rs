use std::sync::Arc;

use parking_lot::RwLock;

use crate::{commands::commands::Command, state::datastate::DataState};

pub struct DeleteCmd {}
impl DeleteCmd {
    pub fn execute(
        data_state: &Arc<RwLock<DataState>>,
        cmd: &Command,
    ) -> Result<Option<Vec<u8>>, String> {
        if cmd.arguments.len() < 1 {
            return Err("Command DELETE requires at least 1 parameter".to_owned());
        } else {
            for key_b in cmd.arguments.iter() {
                let str_key = std::str::from_utf8(&key_b).map_err(|_| "Invalid utf8 key.")?;
                data_state.read().data.remove(str_key);
            }
            return Ok(None);
        }
    }
}
