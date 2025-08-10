use std::{fmt::Debug, hint::unreachable_unchecked};

use crate::class_file_parser::{ClassParser, ContextIndex, ParserContext};
use anyhow::bail;
use jrm_macro::{ClassParser, Getter, constant, constant_enum, define_constants};

#[derive(ClassParser, Default)]
pub struct ConstantPool {
    // meta
    #[class_parser(count(get))]
    #[class_parser(constant_pool(read))]
    constants: Vec<Constant>,
}
impl Debug for ConstantPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        for (index, constant_wrapper) in self.constants.iter().enumerate() {
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
    fn get(&self, index: Self::Idx) -> String {
        self.get_utf8_string(index)
    }
}

impl ConstantPool {
    // TODO safe
    pub fn get_utf8_string(&self, index: u16) -> String {
        if let Constant::Utf8(utf8) = self.constants[index as usize].clone() {
            return String::from(utf8);
        }
        unsafe { unreachable_unchecked() }
    }
    pub fn get_with<F, T>(&self, index: u16, f: F) -> T
    where
        F: FnOnce(&Constant) -> T,
    {
        let constant = &self.constants[index as usize];
        f(constant)
    }
    pub fn get(&self, index: u16) -> Option<&Constant> {
        self.constants.get(index as usize)
    }
}

#[cfg(feature = "test")]
impl From<Vec<Constant>> for ConstantPool {
    fn from(value: Vec<Constant>) -> Self {
        Self { constants: value }
    }
}
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

impl TryFrom<&Constant> for String {
    type Error = anyhow::Error;
    fn try_from(value: &Constant) -> Result<Self, Self::Error> {
        match &value {
            Constant::Utf8(constant_utf8) => Ok(String::from_utf8(constant_utf8.bytes.clone())?),
            _ => bail!("constant is not utf8"),
        }
    }
}

define_constants! {
    pub struct ConstantUtf8 {
        #[class_parser(count(set))]
        length: u16,
        #[class_parser(count(impled))]
         bytes: Vec<u8>,
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
        #[class_parser(constant_index(check))]
        #[getter(copy)]
        name_index: u16,
    }
    pub struct ConstantString {
        #[class_parser(constant_index(check))]
        #[getter(copy)]
        string_index: u16,
    }
    #[constant(__ref)]
    pub struct ConstantFieldRef {}
    #[constant(__ref)]
    pub struct ConstantMethodRef {}
    #[constant(__ref)]
    pub struct ConstantInterfaceMethodRef {}
    pub struct ConstantNameAndType {
        #[class_parser(constant_index(check))]
        #[getter(copy)]
        name_index: u16,

        #[class_parser(constant_index(check))]
        #[getter(copy)]
        descriptor_index: u16,
    }
    pub struct ConstantMethodHandle {
        #[getter(copy)]
        reference_kind: u8,

        #[class_parser(constant_index(check))]
        #[getter(copy)]
        reference_index: u16,
    }
    pub struct ConstantMethodType {
        #[class_parser(constant_index(check))]
        #[getter(copy)]
        descriptor_index: u16,
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

#[cfg(feature = "test")]
impl From<String> for Constant {
    fn from(value: String) -> Self {
        Self::Utf8(value.into())
    }
}

#[cfg(feature = "test")]
impl From<String> for ConstantUtf8 {
    fn from(value: String) -> Self {
        Self {
            tag: 0,
            length: value.len() as u16,
            bytes: value.into(),
        }
    }
}

#[cfg(feature = "test")]
impl ConstantClass {
    pub fn new(name_index: u16) -> Self {
        Self { tag: 0, name_index }
    }
}

#[cfg(feature = "test")]
impl ConstantString {
    pub fn new(string_index: u16) -> Self {
        Self {
            tag: 0,
            string_index,
        }
    }
}

// impl Debug for ConstantUtf8 {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let utf8_string = String::from_utf8_lossy(&self.bytes);
//         write!(f, "{}", utf8_string)
//     }
// }

impl From<ConstantUtf8> for String {
    fn from(value: ConstantUtf8) -> Self {
        unsafe { String::from_utf8_unchecked(value.bytes) }
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        class_file_parser::ContextIndex,
        constant_pool::{Constant, ConstantClass, ConstantPool, ConstantUtf8},
    };

    fn test_constant_pool() -> ConstantPool {
        ConstantPool {
            constants: vec![
                Constant::Class(ConstantClass {
                    tag: 2,
                    name_index: 1,
                }),
                Constant::Utf8(ConstantUtf8 {
                    tag: 99,
                    length: 4,
                    bytes: "aaaa".bytes().collect(),
                }),
            ],
            ..Default::default()
        }
    }
    #[test]
    fn test_constant_pool_index() {
        let constant_pool = test_constant_pool();
        let _ = 0_u16;
        let utf8 = ContextIndex::get(&constant_pool, 1_u16);
        assert_eq!(utf8, "aaaa");
    }
    #[test]
    fn test_constant_pool_get_utf8_string() {
        let constant_pool = test_constant_pool();
        let utf8_string = constant_pool.get_utf8_string(1);
        assert_eq!(utf8_string, "aaaa");
    }
}
