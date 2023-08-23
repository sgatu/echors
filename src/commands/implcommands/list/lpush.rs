use std::sync::Arc;

use crate::{
    commands::commands::Command,
    state::datastate::{DataState, DataType, DataWrapper, ListType, StringType},
};

pub struct LPushCmd {}
impl LPushCmd {
    pub fn execute(data_state: &Arc<DataState>, cmd: &Command) -> Result<Option<Vec<u8>>, String> {
        if cmd.arguments.len() < 2 {
            return Err("Invalid number of arguments for LPUSH command".to_owned());
        }
        let key =
            std::str::from_utf8(cmd.arguments[0]).map_err(|_| "Invalid utf8 key".to_owned())?;
        let values = cmd.arguments.split_at(1).1;
        let opt_key = data_state.get_mut(key);
        if opt_key.is_none() {
            let mut vec_values: Vec<String> = Vec::new();
            for i in 0..values.len() {
                vec_values.push(
                    std::str::from_utf8(values[i])
                        .map_err(|_| format!("Invalid utf8 value at index {}", i).to_owned())?
                        .to_owned(),
                );
            }
            data_state.data.insert(
                key.to_owned(),
                DataWrapper::new(DataType::List(ListType::new(vec_values)), None),
            );
            return Ok(None);
        }
        let mut result = opt_key.unwrap();
        if let DataType::List(ref mut l) = result.value_mut().get_data_mut() {
            for i in 0..values.len() {
                let value = std::str::from_utf8(values[i])
                    .map_err(|_| format!("Invalid utf8 value at index {}", i).to_owned())?;
                let _ = l.push(StringType::new(value.to_owned()));
            }
            return Ok(None);
        }
        return Err("Data at specified key is not a list".to_owned());
    }
}
