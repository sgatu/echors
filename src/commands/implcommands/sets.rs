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
            let new_data = DataType::String(StringType::new(value.to_owned()));
            let _ = rlock.set(key, new_data, expire);
            Ok(None)
            /*let current_data = rlock.get_mut(key);
            if let Some(mut d) = current_data {
                // let new_expire = expire.get_expire(Some(d.value()));
                // let data = d.value_mut();
                // let new_data = new_expire.map_or_else(
                //     || DataWrapper::new(DataType::String(StringType::new(value.to_owned())), None),
                //     |exp_u64| {
                //         DataWrapper::new(
                //             DataType::String(StringType::new(value.to_owned())),
                //             Some(AtomicU64::new(exp_u64)),
                //         )
                //     },
                // );
                let new_expire = expire.get_expire(Some(d.value()));
                let new_data = DataType::String(StringType::new(value.to_owned()));
                data_state.read().data.insert(key.to_owned(), new_data);
                if let Some(exp) = new_expire {
                    data_state.read().expires.insert(key.to_owned(), exp);
                }
                let _ = mem::replace(&mut *data, new_data);
                Ok(None)
            } else {
                let new_expire = expire.get_expire(None);
                let new_data = DataType::String(StringType::new(value.to_owned()));
                data_state.read().data.insert(key.to_owned(), new_data);
                if let Some(exp) = new_expire {
                    data_state.read().expires.insert(key.to_owned(), exp);
                }
                Ok(None)
            }*/
        }
    }
}
