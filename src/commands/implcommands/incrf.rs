use std::sync::Arc;

use parking_lot::RwLock;

use crate::{
    commands::commands::Command,
    state::{
        datastate::{Data, DataState, DataType},
        expires::ExpireParameter,
    },
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
                return Err("Invalid f32 value".to_owned());
            }
            let by_b: [u8; 4] = [
                (*cmd.arguments[1])[0],
                (*cmd.arguments[1])[1],
                (*cmd.arguments[1])[2],
                (*cmd.arguments[1])[3],
            ];
            by = f32::from_le_bytes(by_b);
        }

        let response: Vec<u8>;
        {
            let rlock = data_state.read();
            let old_data = rlock.get_mut(key);
            if let Some(mut d) = old_data {
                let data = d.value_mut().get_data_mut();
                match *data {
                    DataType::Float(ref mut f) => {
                        let curr_val = f.get_mut();
                        *curr_val += by;
                        response = f.serialize().to_vec();
                    }

                    _ => return Err("Invalid type".to_owned()),
                };
            } else {
                let _data = Data::<f32>::new(by);
                response = _data.serialize().to_vec();
                let _ = rlock.set(key, DataType::Float(_data), ExpireParameter::None);
            }
        }
        Ok(Some(response))
    }
}
