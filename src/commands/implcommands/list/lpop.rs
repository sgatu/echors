use std::sync::Arc;

use crate::{
    commands::commands::Command,
    state::datastate::{DataState, DataType},
};

pub struct LPopCmd {}
impl LPopCmd {
    pub fn execute(data_state: &Arc<DataState>, cmd: &Command) -> Result<Option<Vec<u8>>, String> {
        if cmd.arguments.len() < 1 {
            return Err("Invalid number of arguments for LPOP command".to_owned());
        }
        let key =
            std::str::from_utf8(cmd.arguments[0]).map_err(|_| "Invalid utf8 key".to_owned())?;
        let opt_list = data_state.get_mut(key);
        if opt_list.is_none() {
            return Err("Key not found".to_owned());
        }
        let mut value_count: u32 = 1;
        if cmd.arguments.len() == 2 {
            let count_b = cmd.arguments.get(1).unwrap();
            value_count = u32::from_le_bytes([count_b[0], count_b[1], count_b[2], count_b[3]]);
        }
        let mut value_obj = opt_list.unwrap();
        if let DataType::List(list) = value_obj.get_data_mut() {
            let mut result: Vec<u8> = Vec::new();
            for _ in 0..value_count {
                result.extend(list.pop().unwrap().serialize());
            }
            return Ok(Some(result));
        }
        return Err("Data at specified key is not a list".to_owned());
    }
}
