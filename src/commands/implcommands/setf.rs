use std::sync::Arc;

use parking_lot::RwLock;

use crate::{
    commands::commands::Command,
    state::{
        datastate::{Data, DataState, DataType},
        expires::ExpireParameter,
    },
};
pub struct SetF {}
impl SetF {
    pub fn execute(
        data_state: &Arc<RwLock<DataState>>,
        cmd: &Command,
    ) -> Result<Option<Vec<u8>>, String> {
        if cmd.arguments.len() != 2 {
            return Err("Invalid number of arguments for SETF command".to_owned());
        }
        let key =
            std::str::from_utf8(cmd.arguments[0]).map_err(|_| "Invalid utf8 key".to_owned())?;
        if cmd.arguments[1].len() < 4 {
            return Err("Invalid f32 value".to_owned());
        }
        let numb: [u8; 4] = [
            (*cmd.arguments[1])[0],
            (*cmd.arguments[1])[1],
            (*cmd.arguments[1])[2],
            (*cmd.arguments[1])[3],
        ];
        let value: f32 = f32::from_le_bytes(numb);
        {
            let rlock = data_state.read();
            let _ = rlock.set(
                key,
                DataType::Float(Data::<f32>::new(value)),
                ExpireParameter::None,
            );
            Ok(None)
        }
    }
}
