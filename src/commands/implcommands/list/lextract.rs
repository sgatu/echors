use std::sync::Arc;

use parking_lot::RwLock;

use crate::{
    commands::commands::Command,
    state::datastate::{DataState, DataType},
};

pub struct LExtractCmd {}
impl LExtractCmd {
    pub fn execute(
        data_state: &Arc<RwLock<DataState>>,
        cmd: &Command,
    ) -> Result<Option<Vec<u8>>, String> {
        if cmd.arguments.len() != 3 {
            return Err("Invalid number of arguments for LRANGE command".to_owned());
        }
        let key =
            std::str::from_utf8(cmd.arguments[0]).map_err(|_| "Invalid utf8 key".to_owned())?;
        let rlock = data_state.read();
        let opt_list = rlock.get_mut(key);
        if opt_list.is_none() {
            return Err("Key not found".to_owned());
        }
        let start_b = cmd.arguments.get(1).unwrap();
        let start_pos = u32::from_le_bytes([start_b[0], start_b[1], start_b[2], start_b[3]]);
        let end_b = cmd.arguments.get(2).unwrap();
        let end_pos = u32::from_le_bytes([end_b[0], end_b[1], end_b[2], end_b[3]]);
        let mut value_obj = opt_list.unwrap();
        if let DataType::List(list) = value_obj.value_mut().get_data_mut() {
            return Ok(Some(list.srlz_extract_range_with_start(
                start_pos as usize,
                end_pos as usize,
            )));
        }
        return Err("Data at specified key is not a list".to_owned());
    }
}
