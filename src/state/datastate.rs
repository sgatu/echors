use dashmap::DashMap;
use std::cmp;
use string_builder::ToBytes;
pub struct DataState {
    pub data: DashMap<String, DataType>,
}
#[repr(u8)]
pub enum DataTypeByte {
    Integer = 1,
    Float = 2,
    String = 3,
    StrList = 4,
    Map = 5,
    HLL = 6,
}
pub struct Data<T> {
    data: T,
}

pub type StringType = Data<Vec<u8>>;
pub type ListType = Data<Vec<StringType>>;
pub type IntType = Data<i32>;
pub type FloatType = Data<f32>;

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
        println!(
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
