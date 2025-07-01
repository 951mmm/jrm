use std::fmt::Debug;

use crate::class_file_parser::{ClassLookUpParser, ClassParser};
use crate::class_reader::ClassReader;
use jrm_macro::ClassParser;
#[derive(ClassParser)]
#[sized_wrapper]
pub struct ConstantPool(
    #[lookup_outer]
    #[impl_sized]
    #[constant_pool]
    pub Vec<ConstantWrapper>,
);
impl Debug for ConstantPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        for (index, constant_wrapper) in self.0.iter().enumerate() {
            if index == 0 {
                continue; // Skip index 0 as it is reserved
            }
            writeln!(f, "#{}: {:?}", index, constant_wrapper)?;
        }
        Ok(())
    }
}

#[derive(Debug, ClassParser)]
pub struct ConstantWrapper {
    pub tag: u8,
    #[with_lookup(tag)]
    pub constant: Constant,
}

#[derive(Debug, ClassParser)]
#[repr(u8)]
pub enum Constant {
    Utf8(ConstantUtf8) = 1,
    Integer(i32) = 3,
    Float(f32),
    Long(i64),
    Double(f64),
    Class(u16),
    String(u16),
    FieldRef(u16, u16),
    MethodRef(u16, u16),
    InterfaceMethodRef(u16, u16),
    NameAndType(u16, u16),
    MethodHandle(u8, u16) = 15,
    MethodType(u16),
    Dynamic(u16, u16),
    InvokeDynamic(u16, u16),
    Module(u16),
    Package(u16),
    Invalid = 0,
}

#[derive(ClassParser)]
pub struct ConstantUtf8 {
    pub length: u16,
    #[with_lookup(length)]
    pub bytes: Vec<u8>,
}

impl Debug for ConstantUtf8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let utf8_string = String::from_utf8_lossy(&self.bytes);
        write!(f, "{}", utf8_string)
    }
}
