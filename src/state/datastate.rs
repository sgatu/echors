use std::collections::HashMap;

use parking_lot::RwLock;

pub struct DataState {
    pub data: RwLock<HashMap<String, RwLock<DataType>>>,
}
pub enum DataType {
    Number([u8; 4]),
    String(String),
    List(Vec<String>),
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
