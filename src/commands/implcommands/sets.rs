use std::sync::Arc;

use parking_lot::RwLock;

use crate::{
    commands::commands::Command,
    state::{
        datastate::{DataState, DataType, StringType},
        expires::ExpireParameter,
    },
};

pub struct SetSCmd {}
impl SetSCmd {
    pub fn execute(
        data_state: &Arc<RwLock<DataState>>,
        cmd: &Command,
    ) -> Result<Option<Vec<u8>>, String> {
        if cmd.arguments.len() < 2 {
            return Err("Invalid number of arguments for SETS command".to_owned());
        }
        let mut expire: ExpireParameter = ExpireParameter::None;

        if cmd.arguments.len() > 2 {
            if cmd.arguments.len() > 3 {
                return Err("Invalid number of arguments for SETS command".to_owned());
            }
            expire = ExpireParameter::from(cmd.arguments[2]);
        }

        let key =
            std::str::from_utf8(cmd.arguments[0]).map_err(|_| "Invalid utf8 key".to_owned())?;
        let value =
            std::str::from_utf8(cmd.arguments[1]).map_err(|_| "Invalid utf8 value".to_owned())?;
        {
            let rlock = data_state.read();
            let _ = rlock.set(
                key,
                DataType::String(StringType::new(value.to_owned())),
                expire,
            );
            Ok(None)
        }
    }
}
