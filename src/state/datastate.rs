use crate::data::HLL;
use dashmap::{
    mapref::one::{Ref, RefMut},
    DashMap,
};
use log::debug;
use std::{
    cmp, mem,
    sync::atomic::{AtomicU32, AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
use string_builder::ToBytes;

use super::expires::{ExpireParameter, NO_EXPIRE};

#[repr(u8)]
pub enum DataTypeByte {
    Integer = 1,
    Float = 2,
    String = 3,
    StrList = 4,
    Map = 5,
    Long = 6,
}
pub struct Data<T> {
    data: T,
}

pub type StringType = Data<Vec<u8>>;
pub type ListType = Data<Vec<StringType>>;
pub type IntType = Data<i32>;
pub type FloatType = Data<f32>;
pub type HLLType = Data<HLL>;

impl IntType {
    pub fn new(num: i32) -> Self {
        Self { data: num }
    }

    pub fn to_bytes(&self) -> [u8; 4] {
        return i32::to_le_bytes(self.data);
    }

    pub fn serialize(&self) -> [u8; 5] {
        let bytes = self.to_bytes();
        let full_serialized: [u8; 5] = [
            DataTypeByte::Integer as u8,
            bytes[0],
            bytes[1],
            bytes[2],
            bytes[3],
        ];
        return full_serialized;
    }
}
impl FloatType {
    pub fn new(num: f32) -> Self {
        Self { data: num }
    }
    pub fn to_bytes(&self) -> [u8; 4] {
        return f32::to_le_bytes(self.data);
    }

    pub fn serialize(&self) -> [u8; 5] {
        let bytes = self.to_bytes();
        let full_serialized: [u8; 5] = [
            DataTypeByte::Float as u8,
            bytes[0],
            bytes[1],
            bytes[2],
            bytes[3],
        ];
        return full_serialized;
    }
}

// Str list
impl ListType {
    pub fn new(elements: Vec<String>) -> Self {
        let mut result: Vec<StringType> = Vec::new();
        for e in elements {
            result.push(Data::<Vec<u8>>::new(e));
        }
        Self { data: result }
    }
    pub fn srlz_range(&mut self, end: usize) -> Vec<u8> {
        return self.srlz_extract_range_with_start(0, end);
    }
    pub fn srlz_range_with_start(&self, start: usize, end: usize) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        result.push(DataTypeByte::StrList as u8);
        let start_cmp = cmp::min(start, self.data.len());
        let end_cmp = cmp::min(end, self.data.len());
        debug!(
            "Range with start: start = {} | {}, end = {} | {}, listLen: {}",
            start,
            start_cmp,
            end,
            end_cmp,
            self.data.len()
        );
        if end_cmp <= start_cmp {
            return result;
        }

        for i in start_cmp..end_cmp {
            result.extend(self.data[i].serialize());
        }
        return result;
    }
    pub fn push(&mut self, data: StringType) -> Result<(), ()> {
        self.data.push(data);
        return Ok(());
    }
    pub fn pop(&mut self) -> Option<StringType> {
        return self.data.pop();
    }
    pub fn srlz_extract_range(&mut self, end: usize) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        result.push(DataTypeByte::StrList as u8);
        let mut max = cmp::min(end, self.data.len());
        while max > 0 {
            let v = self.data.remove(0);
            let val = v.serialize();
            result.extend(val);
            max -= 1;
        }
        return result;
    }
    pub fn srlz_extract_range_with_start(&mut self, start: usize, end: usize) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        result.push(DataTypeByte::StrList as u8);
        let start_cmp = cmp::min(start, self.data.len());
        let end_cmp = cmp::min(end, self.data.len());
        if end_cmp <= start_cmp {
            return result;
        }
        let mut max = start_cmp;
        while max < end_cmp {
            result.extend(self.data.remove(start_cmp).serialize());
            max += 1;
        }
        return result;
    }
    pub fn srlz_len(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        result.push(DataTypeByte::Integer as u8);
        result.append(&mut u32::to_le_bytes(self.data.len() as u32).to_vec());
        return result;
    }
}
impl StringType {
    pub fn new(str: String) -> Self {
        let mut asvecdata: Vec<u8> = Vec::new();
        let data_type = DataTypeByte::String as u8;
        asvecdata.push(data_type);
        let sz = str.len() as u32;
        asvecdata.extend(sz.to_le_bytes());
        asvecdata.extend(str.to_bytes().iter());
        Self { data: asvecdata }
    }

    pub fn serialize(&self) -> &Vec<u8> {
        return &self.data;
    }
}
impl HLLType {
    pub fn new() -> Self {
        Self { data: HLL::new(14) }
    }
    pub fn new_from_hll(hll: HLL) -> Self {
        return Self { data: hll };
    }
    pub fn srlz_count(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        let count = self.data.count();
        if count < u32::MAX as u64 {
            result.push(DataTypeByte::Integer as u8);
            result.append(&mut u32::to_le_bytes(count as u32).to_vec());
            return result;
        }
        result.push(DataTypeByte::Long as u8);
        result.append(&mut u64::to_le_bytes(count).to_vec());
        return result;
    }
}
impl<T> Data<T> {
    pub fn get(&self) -> &T {
        return &self.data;
    }
    pub fn get_mut(&mut self) -> &mut T {
        return &mut self.data;
    }
}

pub enum DataType {
    Int(IntType),
    Float(FloatType),
    String(StringType),
    List(ListType),
    HLL(HLLType),
}
pub struct DataState {
    pub data: DashMap<String, DataWrapper>,
    removed_count: AtomicU32,
    last_expired_cleanup: AtomicU64,
}
impl DataState {
    const MIN_TIME_BETWEEN_CLEANING: u64 = 100000;
    pub fn new() -> Self {
        Self {
            data: DashMap::new(),
            removed_count: AtomicU32::new(0),
            last_expired_cleanup: AtomicU64::new(0),
        }
    }
    pub fn remove(&self, key: &str) {
        let old = self.data.remove(key);
        if let Some(_) = old {
            self.removed_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn remove_all(&self, keys: Vec<&str>) {
        let mut removed: u32 = 0;
        for key in keys {
            let old = self.data.remove(key);
            if let Some(_) = old {
                removed += 1;
            }
        }
        if removed > 0 {
            debug!("Removed {} keys", removed);
            self.removed_count.fetch_add(removed, Ordering::Relaxed);
        }
    }
    pub fn remove_all_string(&self, keys: Vec<String>) {
        let mut removed: u32 = 0;
        for key in keys {
            let old = self.data.remove(&key);
            if let Some(_) = old {
                removed += 1;
            }
        }
        if removed > 0 {
            debug!("Removed {} keys", removed);
            self.removed_count.fetch_add(removed, Ordering::Relaxed);
        }
    }
    pub fn maintenance_work(&self) {
        let current_removed = self.removed_count.load(Ordering::SeqCst);
        if current_removed as f32 / self.data.len() as f32 > 0.1f32 || current_removed > 50000 {
            debug!("Hashmap shrinking...");
            self.data.shrink_to_fit();
            self.removed_count.store(0, Ordering::SeqCst);
        }
        let last_expired_cleanup = self.last_expired_cleanup.load(Ordering::SeqCst);
        let current_ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        if (current_ts - last_expired_cleanup) > Self::MIN_TIME_BETWEEN_CLEANING
            || last_expired_cleanup == 0
        {
            let mut rm_keys: Vec<String> = Vec::new();
            self.data.iter().for_each(|kv| {
                let exp = kv.get_expire();
                if let Some(ex) = exp {
                    if ex.load(Ordering::Relaxed) < current_ts {
                        rm_keys.push(kv.key().clone());
                    }
                }
            });
            if rm_keys.len() > 0 {
                self.remove_all_string(rm_keys);
            }
            self.last_expired_cleanup
                .store(current_ts, Ordering::SeqCst);
        }
    }
    //this will get the value if exists and not expired, it also deletes the value if expired and returns None
    pub fn get(&self, key: &str) -> Option<Ref<'_, String, DataWrapper>> {
        {
            let data = self.data.get(key);
            if let None = data {
                return None;
            }
            let wrapper = data.unwrap();
            let expire = wrapper.get_expire();

            if let Some(e) = expire {
                let current = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;
                if e.load(Ordering::Relaxed) > current {
                    return Some(wrapper);
                }
            } else {
                return Some(wrapper);
            }
        }
        // if key exists but expire check didn't return early
        self.remove(key);
        return None;
    }
    //same as above but mut
    pub fn get_mut(&self, key: &str) -> Option<RefMut<'_, String, DataWrapper>> {
        {
            let data = self.data.get_mut(key);
            if let None = data {
                return None;
            }
            let wrapper = data.unwrap();
            let expire = wrapper.get_expire();
            if let Some(e) = expire {
                let current = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;
                if e.load(Ordering::Relaxed) > current {
                    return Some(wrapper);
                }
            } else {
                return Some(wrapper);
            }
        }
        // if key exists but expire check didn't return early
        self.remove(key);
        return None;
    }
    pub fn set(&self, key: &str, value: DataType, expire: ExpireParameter) -> Result<(), ()> {
        let current_data = self.get_mut(key);

        if let Some(mut d) = current_data {
            let new_expire = expire.calc_new_expire(Some(&*d));
            let data = DataWrapper::new(
                value,
                new_expire.map_or_else(|| None, |e| Some(AtomicU64::new(e))),
            );
            let _ = mem::replace(&mut *d, data);
            Ok(())
        } else {
            let new_expire = expire.calc_new_expire(None);
            let data = DataWrapper::new(
                value,
                new_expire.map_or_else(|| None, |e| Some(AtomicU64::new(e))),
            );
            self.data.insert(key.to_owned(), data);
            Ok(())
        }
    }

    pub fn flush(&mut self) {
        self.data = DashMap::new();
    }
}
pub struct DataWrapper {
    data: DataType,
    expire: AtomicU64,
}
impl DataWrapper {
    pub fn new(data: DataType, expire: Option<AtomicU64>) -> Self {
        let exp: AtomicU64;
        if let Some(e) = expire {
            exp = (e.load(Ordering::Relaxed) == 0)
                .then_some(NO_EXPIRE)
                .unwrap_or(e);
        } else {
            exp = NO_EXPIRE;
        }

        Self {
            data: data,
            expire: exp,
        }
    }
    pub fn get_data_mut(&mut self) -> &mut DataType {
        return &mut self.data;
    }
    pub fn get_data(&self) -> &DataType {
        return &self.data;
    }
    pub fn get_expire(&self) -> Option<&AtomicU64> {
        if self.expire.load(Ordering::Relaxed) == 0 {
            None
        } else {
            Some(&self.expire)
        }
    }
}
