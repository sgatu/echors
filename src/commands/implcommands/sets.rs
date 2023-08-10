use std::sync::Arc;

use crate::{
    commands::commands::Command,
    state::datastate::{Data, DataState, DataType},
};
use std::mem;

pub struct SetSCmd {}
impl SetSCmd {
    pub fn execute(data_state: &Arc<DataState>, cmd: &Command) -> Result<Option<Vec<u8>>, String> {
        if cmd.arguments.len() != 2 {
            return Err("Invalid number of arguments for SETS command".to_owned());
        }
        let key =
            std::str::from_utf8(cmd.arguments[0]).map_err(|_| "Invalid utf8 key".to_owned())?;
        let value =
            std::str::from_utf8(cmd.arguments[1]).map_err(|_| "Invalid utf8 value".to_owned())?;
        {
            if !data_state.data.contains_key(key) {
                data_state.data.insert(
                    key.to_owned(),
                    DataType::String(Data::<Vec<u8>>::new(value.to_owned())),
                );
                Ok(None)
            } else {
                let mut result = data_state.data.get_mut(key).unwrap();
                let data = result.value_mut();
                let _ = mem::replace(
                    &mut *data,
                    DataType::String(Data::<Vec<u8>>::new(value.to_owned())),
                );
                Ok(None)
            }
        }
    }
}
