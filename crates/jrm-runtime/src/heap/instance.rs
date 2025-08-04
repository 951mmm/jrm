use jrm_parse::instance_klass::FieldAccessFlags;

use crate::{
    Type,
    heap::{ObjectHeader, ObjectRef},
};

// TODO 紧凑内存布局实现？由于堆没有使用连续空间，因此不需要紧凑内存布局。
/// java instsance,保存在堆上
#[derive(Debug)]
pub struct Instance {
    header: ObjectHeader,
    fields: Vec<Field>,
}

impl Instance {}

#[derive(Debug)]
pub struct Field {
    name: String,
    descriptor: Type,
    value: FieldValue,
    is_static: bool,
    access_flag: FieldAccessFlags,
}

#[derive(Debug)]
pub enum FieldValue {
    Boolean(bool),
    Byte(i8),
    Char(u16),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Ref(ObjectRef),
    // TODO 数组的实现
}
