use std::time::SystemTime;

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
    pub fn get_expire(&self, current_expire: Option<u64>) -> Option<u64> {
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
                if let Some(e) = current_expire {
                    return Some(e);
                } else {
                    None
                }
            }
            ExpireParameter::None => None,
        };
        return out;
    }
}
//pub const NO_EXPIRE: AtomicU64 = AtomicU64::new(0);
