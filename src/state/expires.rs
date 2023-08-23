use std::{marker::PhantomData, sync::Arc, time::SystemTime};

use lazy_static::lazy_static;

use super::datastate::DataWrapper;

pub enum ExpireParameter {
    EXPIREAT(u64),
    EXPIREIN(u32),
    KEEPTTL,
    None,
}
impl ExpireParameter {
    pub fn from(data_ref: &[u8]) -> Self {
        match data_ref {
            [a, b, c, d, e, f, g, h] if data_ref.len() == 8 => {
                ExpireParameter::EXPIREAT(u64::from_le_bytes([*a, *b, *c, *d, *e, *f, *g, *h]))
            }
            [a, b, c, d] if data_ref.len() == 4 => {
                ExpireParameter::EXPIREIN(u32::from_le_bytes([*a, *b, *c, *d]))
            }
            [_] if data_ref.len() == 1 => ExpireParameter::KEEPTTL,
            _ => ExpireParameter::None,
        }
    }
    pub fn get_expire(&self, old_data: Option<&DataWrapper>) -> Option<u64> {
        let out = match self {
            ExpireParameter::EXPIREAT(eat) => Some(*eat),
            ExpireParameter::EXPIREIN(ein) => {
                let time = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;
                return Some(time + *ein as u64);
            }
            ExpireParameter::KEEPTTL => {
                if let Some(data) = old_data {
                    return data.get_expire().read_value();
                } else {
                    None
                }
            }
            ExpireParameter::None => None,
        };
        return out;
    }
}
//value does not matter, just initialization, 0 is just ok as any other value
pub struct ExpirePtr<'a> {
    ptr: *const u64,
    phantom: PhantomData<&'a u64>,
}
impl<'a> ExpirePtr<'a> {
    pub fn new(exp: u64) -> Self {
        Self {
            ptr: exp as *const u64,
            phantom: PhantomData,
        }
    }
    pub fn newc(exp: *const u64) -> Self {
        Self {
            ptr: exp,
            phantom: PhantomData,
        }
    }
    pub fn read_value(&self) -> Option<u64> {
        if self.ptr.is_null() {
            return None;
        }
        return Some(unsafe { *self.ptr });
    }
}
lazy_static! {
    pub static ref EXPIRE_NULL: Arc<ExpirePtr<'static>> =
        Arc::new(ExpirePtr::newc(std::ptr::null()));
}

unsafe impl<'a> Send for ExpirePtr<'a> {}
unsafe impl<'a> Sync for ExpirePtr<'a> {}
