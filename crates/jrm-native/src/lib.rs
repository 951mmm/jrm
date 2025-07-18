pub type JBoolean = bool;
pub type JByte = i8;
pub type JChar = u16;
pub type JShort = i16;
pub type JInt = i32;
pub type JLong = i64;
pub type JSize = i32;
pub type JFloat = f32;
pub type JDouble = f64;

// 指向accessoop的指针
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

pub struct NativeContext {
    pub get_intern_string: Box<dyn Fn(&NativeContext, JObject) -> JObject>,
}

#[cfg(test)]
mod tests {
    use crate::{JObject, NativeContext};

    #[test]
    fn test_ctx() {
        fn get_intern_string(ctx: &NativeContext, this: JObject) -> JObject {
            JObject(1)
        }
        let ctx = NativeContext {
            get_intern_string: Box::new(get_intern_string),
        };
    }
}
