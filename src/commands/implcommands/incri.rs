use std::sync::Arc;

use crate::{
    commands::commands::Command,
    state::datastate::{Data, DataState, DataType},
};
pub struct IncrI {}
impl IncrI {
    pub fn execute(data_state: &Arc<DataState>, cmd: &Command) -> Result<Option<Vec<u8>>, String> {
        if cmd.arguments.len() < 1 {
            return Err("Invalid number of arguments for INCRI command".to_owned());
        }
        let key =
            std::str::from_utf8(cmd.arguments[0]).map_err(|_| "Invalid utf8 key".to_owned())?;
        let mut by: i32 = 1;
        if cmd.arguments.len() > 1 {
            if cmd.arguments[1].len() < 4 {
                return Err("Invalid u32 value".to_owned());
            }
            let by_b: [u8; 4] = [
                (*cmd.arguments[1])[0],
                (*cmd.arguments[1])[1],
                (*cmd.arguments[1])[2],
                (*cmd.arguments[1])[3],
            ];
            by = i32::from_le_bytes(by_b);
        }

        {
            let response: Vec<u8>;
            if !data_state.data.contains_key(key) {
                {
                    let _data = Data::<i32>::new(by);
                    response = _data.serialize().to_vec();
                    // we set value to incryBy if none was specified
                    data_state.data.insert(key.to_owned(), DataType::Int(_data));
                }
                Ok(Some(response))
            } else {
                let mut result = data_state.data.get_mut(key).unwrap();
                let data = result.value_mut();
                match *data {
                    DataType::Int(ref mut i) => {
                        let curr_val = i.get_mut();
                        *curr_val += by;
                        response = i.serialize().to_vec();
                    }

                    _ => return Err("Invalid type".to_owned()),
                };
                Ok(Some(response))
            }
        }
    }
}
