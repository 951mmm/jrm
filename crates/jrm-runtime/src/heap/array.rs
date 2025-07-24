use crate::heap::{ObjectHeader, ObjectRef};

#[derive(Debug)]
pub struct Array {
    header: ObjectHeader,
    length: i32,
    data: ArrayValue,
}

impl Array {
    pub fn new(length: i32, data: ArrayValue) -> Self {
        Self {
            header: Default::default(),
            length,
            data,
        }
    }
}

#[derive(Debug)]
pub enum ArrayValue {
    Boolean(Vec<bool>),
    Byte(Vec<i8>), // 同时用于boolean/byte
    Char(Vec<u16>),
    Short(Vec<i16>),
    Int(Vec<i32>),
    Long(Vec<i64>),
    Float(Vec<f32>),
    Double(Vec<f64>),
    Ref(Vec<ObjectRef>),
}
