use std::sync::Arc;

use parking_lot::RwLock;

use crate::{
    commands::commands::Command,
    state::datastate::{DataState, DataType},
};

pub struct HLLResetCmd {}
impl HLLResetCmd {
    pub fn execute(
        data_state: &Arc<RwLock<DataState>>,
        cmd: &Command,
    ) -> Result<Option<Vec<u8>>, String> {
        if cmd.arguments.len() != 1 {
            return Err("Invalid number of arguments for HLLRESET command".to_owned());
        }
        let key =
            std::str::from_utf8(cmd.arguments[0]).map_err(|_| "Invalid utf8 key".to_owned())?;
        let rlock = data_state.read();
        let opt_key = rlock.get_mut(key);
        if opt_key.is_none() {
            return Err("Key not found".to_owned());
        }
        let mut result = opt_key.unwrap();
        let result_val = result.value_mut().get_data_mut();
        if let DataType::HLL(list) = result_val {
            list.get_mut().reset();
            return Ok(None);
        }
        return Err("Data at specified key is not a valid HLL".to_owned());
    }
}
