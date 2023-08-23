use std::sync::Arc;

use crate::{
    commands::commands::Command,
    data::HLL,
    state::datastate::{DataState, DataType, DataWrapper, HLLType},
};

pub struct HLLAddCmd {}
impl HLLAddCmd {
    pub fn execute(data_state: &Arc<DataState>, cmd: &Command) -> Result<Option<Vec<u8>>, String> {
        if cmd.arguments.len() < 2 {
            return Err("Invalid number of arguments for HLLADD command".to_owned());
        }
        let key =
            std::str::from_utf8(cmd.arguments[0]).map_err(|_| "Invalid utf8 key".to_owned())?;
        let values = cmd.arguments.split_at(1).1;
        let opt_key = data_state.get_mut(key);
        if opt_key.is_none() {
            let mut hll: HLL = HLL::new(14);

            for i in 0..values.len() {
                hll.add(
                    std::str::from_utf8(values[i])
                        .map_err(|_| format!("Invalid utf8 value at index {}", i).to_owned())?,
                );
            }
            data_state.data.insert(
                key.to_owned(),
                DataWrapper::new(DataType::HLL(HLLType::new_from_hll(hll)), None),
            );
            return Ok(None);
        }
        let mut result = opt_key.unwrap();
        if let DataType::HLL(ref mut l) = result.value_mut().get_data_mut() {
            let mut_storage = l.get_mut();
            for i in 0..values.len() {
                let value = std::str::from_utf8(values[i])
                    .map_err(|_| format!("Invalid utf8 value at index {}", i))?;
                mut_storage.add(value);
            }
            return Ok(None);
        }
        return Err("Data at specified key is not a valid HLL".to_owned());
    }
}
