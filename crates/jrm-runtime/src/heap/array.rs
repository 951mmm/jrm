use crate::heap::{ObjectHeader, ObjectRef};

#[derive(Debug)]
pub struct Array {
    header: ObjectHeader,
    length: i32,
    data: ArrayValue,
}

#[derive(Debug)]
pub enum ArrayValue {
    Boolean(Vec<bool>),
    Byte(Vec<i8>), // 同时用于boolean/byte
    Short(Vec<i16>),
    Int(Vec<i32>),
    Long(Vec<i64>),
    Float(Vec<f32>),
    Double(Vec<f64>),
    Ref(Vec<ObjectRef>),
}
