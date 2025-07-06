use std::fmt::Debug;

use crate::class_file_parser::{ClassParser, ContextIndex, ParserContext};
use crate::class_reader::ClassReader;
use anyhow::bail;
use jrm_macro::{ClassParser, constant, constant_enum, define_constants};

#[derive(Clone, ClassParser, Default)]
pub struct ConstantPool(
    #[count(get)]
    #[constant_pool(read)]
    pub Vec<Constant>,
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

impl ContextIndex for ConstantPool {
    type Idx = u16;
    fn get(&self, index: Self::Idx) -> anyhow::Result<String> {
        (&self.0[index as usize]).try_into()
    }
}

impl TryFrom<&Constant> for String {
    type Error = anyhow::Error;
    fn try_from(value: &Constant) -> Result<Self, Self::Error> {
        match &value {
            Constant::Utf8(constant_utf8) => Ok(String::from_utf8(constant_utf8.bytes.clone())?),
            _ => bail!("constant is not utf8"),
        }
    }
}

// #[derive(Clone, Debug, ClassParser)]
// #[enum_entry(get(u8))]
// #[repr(u8)]
// pub enum Constant {
//     Utf8(ConstantUtf8) = 1,
//     Integer(i32) = 3,
//     Float(f32),
//     Long(i64),
//     Double(f64),
//     Class(ConstantClass),
//     String(u16),
//     FieldRef(u16, u16),
//     MethodRef(u16, u16),
//     InterfaceMethodRef(u16, u16),
//     NameAndType(u16, u16),
//     MethodHandle(u8, u16) = 15,
//     MethodType(u16),
//     Dynamic(u16, u16),
//     InvokeDynamic(u16, u16),
//     Module(u16),
//     Package(u16),
//     Invalid = 0,
// }

constant_enum! {
    Utf8,
    Integer,
    Float,
    Long,
    Double,
    Class,
    String,
    FieldRef,
    MethodRef,
    InterfaceMethodRef,
    NameAndType,
    MethodHandle,
    MethodType,
    Dynamic,
    InvokeDynamic,
    Module,
    Package
}

define_constants! {
    pub struct ConstantUtf8 {
        #[count(set)]
        pub length: u16,
        #[count(impled)]
        pub bytes: Vec<u8>,
    }
    #[constant(one_word)]
    pub struct ConstantInteger {}
    #[constant(one_word)]
    pub struct ConstantFloat {}
    #[constant(two_words)]
    pub struct ConstantLong {}
    #[constant(two_words)]
    pub struct ConstantDouble {}
    pub struct ConstantClass {
        #[constant_index(check)]
        pub name_index: u16,
    }
    pub struct ConstantString {
        #[constant_index(check)]
        pub string_index: u16,
    }
    #[constant(__ref)]
    pub struct ConstantFieldRef {}
    #[constant(__ref)]
    pub struct ConstantMethodRef {}
    #[constant(__ref)]
    pub struct ConstantInterfaceMethodRef {}
    pub struct ConstantNameAndType {
        #[constant_index(check)]
        pub name_index: u16,
        #[constant_index(check)]
        pub descriptor_index: u16,
    }
    pub struct ConstantMethodHandle {
        pub reference_kind: u8,
        #[constant_index(check)]
        pub reference_index: u16,
    }
    pub struct ConstantMethodType {
        #[constant_index(check)]
        pub descriptor_index: u16,
    }
    #[constant(dynamic)]
    pub struct ConstantDynamic {}
    #[constant(dynamic)]
    pub struct ConstantInvokeDynamic {}
    #[constant(module)]
    pub struct ConstantModule {}
    #[constant(module)]
    pub struct ConstantPackage {}
}

// impl Debug for ConstantUtf8 {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let utf8_string = String::from_utf8_lossy(&self.bytes);
//         write!(f, "{}", utf8_string)
//     }
// }
#[cfg(test)]
mod tests {

    use crate::{
        class_file_parser::ContextIndex,
        constant_pool::{Constant, ConstantClass, ConstantPool, ConstantUtf8},
    };

    fn test_constant_pool() -> ConstantPool {
        ConstantPool(vec![
            Constant::Class(ConstantClass {
                tag: 2,
                name_index: 12,
            }),
            Constant::Utf8(ConstantUtf8 {
                tag: 99,
                length: 4,
                bytes: "aaaa".bytes().collect(),
            }),
        ])
    }
    #[test]
    fn test_constant_pool_index() {
        let constant_pool = test_constant_pool();
        let i = 0_u16;
        let err = ContextIndex::get(&constant_pool, i).unwrap_err();
        assert_eq!(err.to_string(), "constant is not utf8");
        let utf8 = ContextIndex::get(&constant_pool, 1_u16).unwrap();
        assert_eq!(utf8, "aaaa");
    }
}
