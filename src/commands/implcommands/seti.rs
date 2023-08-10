use std::sync::Arc;

use crate::{
    commands::commands::Command,
    state::datastate::{Data, DataState, DataType},
};
use std::mem;
pub struct SetI {}
impl SetI {
    pub fn execute(data_state: &Arc<DataState>, cmd: &Command) -> Result<Option<Vec<u8>>, String> {
        if cmd.arguments.len() != 2 {
            return Err("Invalid number of arguments for SETI command".to_owned());
        }
        let key =
            std::str::from_utf8(cmd.arguments[0]).map_err(|_| "Invalid utf8 key".to_owned())?;
        if cmd.arguments[1].len() < 4 {
            return Err("Invalid u32 value".to_owned());
        }
        let numb: [u8; 4] = [
            (*cmd.arguments[1])[0],
            (*cmd.arguments[1])[1],
            (*cmd.arguments[1])[2],
            (*cmd.arguments[1])[3],
        ];
        let value: i32 = i32::from_le_bytes(numb);
        {
            if !data_state.data.contains_key(key) {
                {
                    data_state
                        .data
                        .insert(key.to_owned(), DataType::Int(Data::<i32>::new(value)));
                }
                Ok(None)
            } else {
                let mut result = data_state.data.get_mut(key).unwrap();
                let data = result.value_mut();
                let _ = mem::replace(&mut *data, DataType::Int(Data::<i32>::new(value)));
                Ok(None)
            }
        }
    }
}
