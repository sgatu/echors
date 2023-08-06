use std::sync::Arc;

use crate::{
    commands::commands::Command,
    state::datastate::{DataState, DataType},
};

pub struct GetCmd {}
impl GetCmd {
    pub fn execute<'a>(
        data_state: &Arc<DataState>,
        cmd: &Command,
    ) -> Result<Option<Vec<u8>>, String> {
        if cmd.arguments.len() != 1 {
            return Err("Command GET requires 1 paramter".to_owned());
        } else {
            let key = std::str::from_utf8(cmd.arguments[0]).map_err(|_| "Invalid utf8 key.")?;
            let read_state = data_state.data.read();
            if !read_state.contains_key(key) {
                Err("Key not found".to_owned())
            } else {
                let val_lock = read_state.get(key).unwrap();
                let val_read = val_lock.read();
                let result = match &*val_read {
                    DataType::String(v) => Ok(Some(v.to_bytes().to_vec())),
                    DataType::Int(v) => Ok(Some((&v.to_bytes()).to_vec())),
                    DataType::Float(v) => Ok(Some((&v.to_bytes()).to_vec())),
                    DataType::List(_) => Err("Cannot get list".to_owned()),
                };
                return result;
            }
        }
    }
}
