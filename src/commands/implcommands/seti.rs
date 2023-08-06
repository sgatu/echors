use std::{cell::RefCell, sync::Arc};

use parking_lot::{Mutex, RwLock};

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
            let read_state = data_state.data.read();

            if !read_state.contains_key(key) {
                drop(read_state);
                {
                    let mut write_state = data_state.data.write();

                    write_state.insert(
                        key.to_owned(),
                        RwLock::new(DataType::Int(Mutex::new(Data::<i32>::new(value)))),
                    );
                }
                Ok(None)
            } else {
                let mut value_lock = read_state.get(key).unwrap().write();
                let _ = mem::replace(
                    &mut *value_lock,
                    DataType::Int(Mutex::new(Data::<i32>::new(value))),
                );
                Ok(None)
            }
        }
    }
}
