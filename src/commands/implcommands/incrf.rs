use std::sync::Arc;

use parking_lot::RwLock;

use crate::{
    commands::commands::Command,
    state::datastate::{Data, DataState, DataType, DataWrapper},
};
pub struct IncrF {}
impl IncrF {
    pub fn execute(
        data_state: &Arc<RwLock<DataState>>,
        cmd: &Command,
    ) -> Result<Option<Vec<u8>>, String> {
        if cmd.arguments.len() < 1 {
            return Err("Invalid number of arguments for INCRI command".to_owned());
        }
        let key =
            std::str::from_utf8(cmd.arguments[0]).map_err(|_| "Invalid utf8 key".to_owned())?;
        let mut by: f32 = 1.0;
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
            by = f32::from_le_bytes(by_b);
        }

        {
            let response: Vec<u8>;
            if !data_state.read().data.contains_key(key) {
                {
                    let _data = Data::<f32>::new(by);
                    response = _data.serialize().to_vec();
                    // we set value to incryBy if none was specified
                    data_state.read().data.insert(
                        key.to_owned(),
                        DataWrapper::new(DataType::Float(_data), None),
                    );
                }
                Ok(Some(response))
            } else {
                let rlock = data_state.read();
                let mut result = rlock.get_mut(key).unwrap();
                let data = result.value_mut().get_data_mut();
                match data {
                    DataType::Float(ref mut i) => {
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
