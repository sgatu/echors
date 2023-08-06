use std::sync::Arc;

use parking_lot::RwLock;

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
            let read_state = data_state.data.read();

            if !read_state.contains_key(key) {
                drop(read_state);
                {
                    let mut write_state = data_state.data.write();

                    write_state.insert(
                        key.to_owned(),
                        RwLock::new(DataType::String(Data::<String>::new(value.to_owned()))),
                    );
                }
                Ok(None)
            } else {
                let mut value_lock = read_state.get(key).unwrap().write();
                let _ = mem::replace(
                    &mut *value_lock,
                    DataType::String(Data::<String>::new(value.to_owned())),
                );
                Ok(None)
            }
        }
    }
}
