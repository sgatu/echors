use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::SystemTime,
};

use super::datastate::{DataType, DataWrapper};

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
                    let expire = data.get_expire();
                    if expire.isnull {
                        None
                    } else {
                        let time = expire.ptr.load(Ordering::Relaxed);
                        Some(time)
                    }
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
const NULL_PTR: AtomicU64 = AtomicU64::new(0);
pub struct ExpirePtr {
    ptr: AtomicU64,
    isnull: bool,
}

impl ExpirePtr {
    pub fn new(exp: u64) -> Self {
        Self {
            ptr: AtomicU64::new(exp),
            isnull: false,
        }
    }
    pub fn is_null(&self) -> bool {
        return self.isnull;
    }
    pub fn read_value(&self) -> u64 {
        return self.ptr.load(Ordering::Relaxed);
    }
    //keep it private so it's singleton-ish
    const fn null_new() -> Self {
        Self {
            ptr: NULL_PTR,
            isnull: true,
        }
    }
}
pub const EXPIRE_NULL: ExpirePtr = ExpirePtr::null_new();
//bit crazy, maybe, but it's supposed that EXPIRE_NULL is const so it won't change, and others will be behind lock
unsafe impl Send for ExpirePtr {}
unsafe impl Sync for ExpirePtr {}
