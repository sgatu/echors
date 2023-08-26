use std::sync::Arc;

use parking_lot::RwLock;

use crate::{
    commands::commands::Command,
    data::HLL,
    state::{
        datastate::{DataState, DataType, HLLType},
        expires::ExpireParameter,
    },
};

pub struct HLLAddCmd {}
impl HLLAddCmd {
    pub fn execute(
        data_state: &Arc<RwLock<DataState>>,
        cmd: &Command,
    ) -> Result<Option<Vec<u8>>, String> {
        if cmd.arguments.len() < 2 {
            return Err("Invalid number of arguments for HLLADD command".to_owned());
        }
        let key =
            std::str::from_utf8(cmd.arguments[0]).map_err(|_| "Invalid utf8 key".to_owned())?;
        let values = cmd.arguments.split_at(1).1;
        let rlock = data_state.read();
        let opt_key = rlock.get_mut(key);
        if opt_key.is_none() {
            let mut hll: HLL = HLL::new(14);

            for i in 0..values.len() {
                hll.add(
                    std::str::from_utf8(values[i])
                        .map_err(|_| format!("Invalid utf8 value at index {}", i).to_owned())?,
                );
            }
            let _ = data_state.read().set(
                key,
                DataType::HLL(HLLType::new_from_hll(hll)),
                ExpireParameter::None,
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
