use std::fmt::{Debug, Display};

use jrm_macro::ConstantConstuctor;

pub struct ConstantPool(Vec<ConstantWrapper>);

impl ConstantPool {
    pub fn with_capacity(capacity: usize) -> Self {
        ConstantPool(Vec::with_capacity(capacity))
    }
    pub fn add(&mut self, constant_wrapper: ConstantWrapper) {
        self.0.push(constant_wrapper);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Debug for ConstantPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, constant_wrapper) in self.0.iter().enumerate() {
            if index == 0 {
                continue; // Skip index 0 as it is reserved
            }
            writeln!(f, "#{}: {}", index, constant_wrapper)?;
        }
        Ok(())
    }
}

pub struct ConstantWrapper {
    pub tag: u8,
    pub constant: Constant,
}

impl Display for ConstantWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "tag: {}, Constant: {}", self.tag, self.constant)
    }
}
#[derive(ConstantConstuctor)]
pub enum Constant {
    Utf8(ConstantUtf8),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Class(u16),
    String(u16),
    #[ignored]
    FieldRef(u16, u16),
    #[ignored]
    MethodRef(u16, u16),
    #[ignored]
    InterfaceMethodRef(u16, u16),
    #[ignored]
    NameAndType(u16, u16),
    MethodHandle(u8, u16),
    MethodType(u16),
    #[ignored]
    Dynamic(u16, u16),
    #[ignored]
    InvokeDynamic(u16, u16),
    Module(u16),
    Package(u16),
    Placeholder,
}

impl Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Constant::Utf8(s) => write!(f, "[Utf8]: {}", s),
            Constant::Integer(i) => write!(f, "[Integer]: {}", i),
            Constant::Float(fl) => write!(f, "[Float]: {}", fl),
            Constant::Long(l) => write!(f, "[Long]: {}", l),
            Constant::Double(d) => write!(f, "[Double]: {}", d),
            Constant::Class(index) => write!(f, "[Class]: {}", index),
            Constant::String(index) => write!(f, "[String]: {}", index),
            Constant::FieldRef(low, high) => write!(f, "[FieldRef]: {:X}:{:X}", low, high),
            Constant::MethodRef(low, high) => write!(f, "[MethodRef]: {:X}:{:X}", low, high),
            Constant::InterfaceMethodRef(low, high) => {
                write!(f, "[InterfaceMethodRef]: {:X}:{:X}", low, high)
            }
            Constant::NameAndType(low, high) => write!(f, "[NameAndType]: {:X}:{:X}", low, high),
            Constant::MethodHandle(kind, index) => {
                write!(f, "[MethodHandle]: Kind: {}, Index: {}", kind, index)
            }
            Constant::MethodType(index) => write!(f, "[MethodType]: {}", index),
            Constant::Dynamic(low, high) => write!(f, "[Dynamic]: {:X}:{:X}", low, high),
            Constant::InvokeDynamic(low, high) => {
                write!(f, "[InvokeDynamic]: {:X}:{:X}", low, high)
            }
            Constant::Module(index) => write!(f, "[Module]: {}", index),
            Constant::Package(index) => write!(f, "[Package]: {}", index),
            Constant::Placeholder => write!(f, "[Placeholder]"),
        }
    }
}

pub struct ConstantUtf8 {
    pub length: u16,
    pub bytes: Vec<u8>,
}

impl Display for ConstantUtf8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let utf8_string = String::from_utf8_lossy(&self.bytes);
        write!(f, "{}", utf8_string)
    }
}
