use std::sync::Arc;

use crate::{
    commands::commands::Command,
    state::datastate::{DataState, DataType},
};

pub struct LLenCmd {}
impl LLenCmd {
    pub fn execute(data_state: &Arc<DataState>, cmd: &Command) -> Result<Option<Vec<u8>>, String> {
        if cmd.arguments.len() != 1 {
            return Err("Invalid number of arguments for LLEN command".to_owned());
        }
        let key =
            std::str::from_utf8(cmd.arguments[0]).map_err(|_| "Invalid utf8 key".to_owned())?;
        let opt_list = data_state.get(key);
        if opt_list.is_none() {
            return Err("Key not found".to_owned());
        }
        let result = opt_list.unwrap();
        let result_val = result.value().get_data();
        if let DataType::List(list) = result_val {
            return Ok(Some(list.srlz_len()));
        }
        return Err("Data at specified key is not a list".to_owned());
    }
}
