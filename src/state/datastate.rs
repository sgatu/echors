use std::collections::HashMap;

use parking_lot::{Mutex, RwLock};

pub struct DataState {
    pub data: RwLock<HashMap<String, RwLock<DataType>>>,
}

pub struct Data<T> {
    data: T,
}

impl Data<i32> {
    pub fn new(num: i32) -> Self {
        Self { data: num }
    }
    pub fn incr(&mut self, by: i32) {
        self.data += by;
    }
    pub fn to_bytes(&self) -> [u8; 4] {
        return i32::to_le_bytes(self.data);
    }
}
impl Data<f32> {
    pub fn new(num: f32) -> Self {
        Self { data: num }
    }
    pub fn incr(&mut self, val: f32) {
        self.data += val;
    }
    pub fn to_bytes(&self) -> [u8; 4] {
        return f32::to_le_bytes(self.data);
    }
}
impl Data<String> {
    pub fn new(str: String) -> Self {
        Self { data: str }
    }
    pub fn to_bytes(&self) -> &[u8] {
        return self.data.as_bytes();
    }
}
impl Data<Vec<String>> {
    pub fn new(list: Vec<String>) -> Self {
        Self { data: list }
    }
    /*pub fn to_bytes(&self) -> [u8; 4] {

    }*/
}
impl<T> Data<T> {
    pub fn get(&self) -> &T {
        return &self.data;
    }
}

pub enum DataType {
    Int(Mutex<Data<i32>>),
    Float(Mutex<Data<f32>>),
    String(Data<String>),
    List(Data<Vec<String>>),
}
impl DataState {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }
}
/*pub struct Data {
    pub data: Vec<u8>,
    pub data_type: Number;
}*/
