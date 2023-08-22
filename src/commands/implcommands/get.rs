use std::sync::Arc;

use crate::{
    commands::commands::Command,
    state::datastate::{DataState, DataType},
};

pub struct GetCmd {}
impl GetCmd {
    pub fn execute(data_state: &Arc<DataState>, cmd: &Command) -> Result<Option<Vec<u8>>, String> {
        if cmd.arguments.len() != 1 {
            return Err("Command GET requires 1 paramter".to_owned());
        } else {
            let key = std::str::from_utf8(cmd.arguments[0]).map_err(|_| "Invalid utf8 key.")?;
            if !data_state.data.contains_key(key) {
                Err("Key not found".to_owned())
            } else {
                //release lock
                {
                    let result = data_state.get(key);
                    if let None = result {
                        return Err("Key not found".to_owned());
                    } else {
                        let uresult = result.unwrap();
                        let value = uresult.value().get_data();
                        match &*value {
                            DataType::String(v) => Ok(Some(v.serialize().to_vec())),

                            DataType::Int(v) => Ok(Some(v.serialize().to_vec())),
                            DataType::Float(v) => Ok(Some(v.serialize().to_vec())),
                            _ => Err("Data type is not simple".to_owned()),
                        }
                    }
                }
            }
        }
    }
}
