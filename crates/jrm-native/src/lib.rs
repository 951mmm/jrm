
pub type JBoolean = bool;
pub type JByte = i8;
pub type JChar = u16;
pub type JShort = i16;
pub type JInt = i32;
pub type JLong = i64;
pub type JSize = i32;
pub type JFloat = f32;
pub type JDouble = f64;

// 指向堆的指针
pub struct JObject(i32);
pub type JClass = JObject;
pub type JThrowable = JObject;
pub type JString = JObject;
pub type JArray = JObject;
pub type JBooleanArray = JArray;
pub type JByteArray = JArray;
pub type JCharArray = JArray;
pub type JShortArray = JArray;
pub type JIntArray = JArray;
pub type JLongArray = JArray;
pub type JFloatArray = JArray;
pub type JDoubleArray = JArray;
pub type JObjectArray = JArray;

// 元数据中field的下标
pub struct JField(i32);

pub enum JValue {
    Boolean(bool),
    Byte(i8),
    Char(u16),
    Short(u16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Object(JObject),
}

// 应该保持无状态
pub struct NativeContext {
    pub get_intern_string: Box<dyn Fn(JObject) -> JObject>,
    pub get_static_field: Box<dyn Fn(&JClass, &str, &str) -> Result<JField>>,
    pub set_static_object_field: Box<dyn Fn(&JClass, JField, JObject)>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0} not found")]
    NotFound(String),
}
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    

    #[test]
    fn test_ctx() {}
}
