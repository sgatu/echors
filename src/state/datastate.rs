use dashmap::DashMap;
use string_builder::ToBytes;

pub struct DataState {
    pub data: DashMap<String, DataType>,
}
#[repr(u8)]
pub enum DataTypeByte {
    Integer = 1,
    Float = 2,
    String = 3,
}
pub struct Data<T> {
    data: T,
}

impl Data<i32> {
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
impl Data<f32> {
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
impl Data<Vec<u8>> {
    pub fn new(str: String) -> Self {
        let mut asvecdata: Vec<u8> = Vec::new();
        let data_type = DataTypeByte::String as u8;
        asvecdata.push(data_type);
        asvecdata.extend(str.to_bytes().iter());
        Self { data: asvecdata }
    }
    pub fn serialize(&self) -> &Vec<u8> {
        return &self.data;
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
    Int(Data<i32>),
    Float(Data<f32>),
    String(Data<Vec<u8>>),
    //List(Data<Vec<String>>),
}
impl DataState {
    pub fn new() -> Self {
        Self {
            data: DashMap::new(),
        }
    }
}

/*pub struct Data {
    pub data: Vec<u8>,
    pub data_type: Number;
}*/
