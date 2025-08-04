use std::sync::Arc;

use bitflags::{Flags, bitflags};

use crate::attributes::Attribute;
use crate::class_file_parser::{ClassParser, ParserContext};
use crate::constant_pool::{ConstantClass, ConstantPool};
use jrm_macro::{ClassParser, Getter, KlassDebug};

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct ClassAccessFlags: u16 {
        const PUBLIC     = 0x0001;
        const FINAL      = 0x0010;
        const SUPER      = 0x0020;
        const INTERFACE  = 0x0200;
        const ABSTRACT   = 0x0400;
        const SYNTHETIC  = 0x1000;
        const ANNOTATION = 0x2000;
        const ENUM       = 0x4000;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct FieldAccessFlags: u16 {
        const PUBLIC     = 0x0001;
        const PRIVATE    = 0x0002;
        const PROTECTED  = 0x0004;
        const STATIC     = 0x0008;
        const FINAL      = 0x0010;
        const VOLATILE   = 0x0040;
        const TRANSIENT  = 0x0080;
        const SYNTHETIC  = 0x1000;
        const ENUM       = 0x4000;
    }
}

bitflags! {
    #[derive(Debug, Default, Clone, Copy)]
    pub struct MethodAccessFlags: u16 {
        const PUBLIC        = 0x0001;
        const PRIVATE       = 0x0002;
        const PROTECTED     = 0x0004;
        const STATIC        = 0x0008;
        const FINAL         = 0x0010;
        const SYNCHRONIZED  = 0x0020;
        const BRIDGE        = 0x0040;
        const VARARGS       = 0x0080;
        const NATIVE        = 0x0100;
        const ABSTRACT      = 0x0400;
        const STRICT        = 0x0800;
        const SYNTHETIC     = 0x1000;
    }
}

macro_rules! impl_class_parser_for_bitflags {
    ($ty: ty, $bits: ty) => {
        impl ClassParser for $ty {
            fn parse(ctx: &mut ParserContext) -> anyhow::Result<Self> {
                let bits = <$bits as ClassParser>::parse(ctx)?;
                Self::from_bits(bits).ok_or(anyhow::anyhow!("invalid flags"))
            }
        }
    };
}

impl_class_parser_for_bitflags!(ClassAccessFlags, u16);
impl_class_parser_for_bitflags!(FieldAccessFlags, u16);
impl_class_parser_for_bitflags!(MethodAccessFlags, u16);

#[derive(KlassDebug, ClassParser, Getter)]
pub struct InstanceKlass {
    #[hex]
    #[getter(skip)]
    magic: u32,

    minor_version: u16,

    major_version: u16,

    #[count(set)]
    #[constant_index(setend)]
    #[getter(skip)]
    constant_pool_count: u16,

    #[constant_pool(set)]
    constant_pool: Arc<ConstantPool>,

    #[hex]
    access_flags: ClassAccessFlags,

    #[constant_index(check)]
    #[getter(copy)]
    this_class: u16,

    #[constant_index(check)]
    #[getter(copy)]
    super_class: u16,

    #[count(set)]
    #[getter(skip)]
    interfaces_count: u16,

    #[count(get)]
    interfaces: Vec<Interface>,

    #[count(set)]
    #[getter(skip)]
    fields_count: u16,

    #[count(get)]
    fields: Vec<Field>,

    #[count(set)]
    #[getter(skip)]
    methods_count: u16,

    #[count(get)]
    methods: Vec<Method>,

    #[count(set)]
    #[getter(skip)]
    attributes_count: u16,

    #[count(impled)]
    attributes: Vec<Attribute>,
}
impl InstanceKlass {
    pub fn parse_from_bytes(bytes: Vec<u8>) -> anyhow::Result<Self> {
        let mut ctx = ParserContext::from(bytes);
        Self::parse(&mut ctx)
    }
}
type Interface = ConstantClass;
#[derive(Debug, ClassParser)]
pub struct Field {
    access_flags: FieldAccessFlags,
    name_index: u16,
    descriptor_index: u16,
    #[count(set)]
    attributes_count: u16,
    #[count(impled)]
    attributes: Vec<Attribute>,
}

#[derive(Debug, ClassParser, Getter)]
pub struct Method {
    access_flags: MethodAccessFlags,
    #[getter(copy)]
    name_index: u16,
    #[getter(copy)]
    descriptor_index: u16,
    #[count(set)]
    #[getter(copy)]
    attributes_count: u16,
    #[count(impled)]
    attributes: Vec<Attribute>,
}

#[cfg(test)]
mod tests {
    use jrm_macro::Getter;

    #[test]
    fn test_getter() {
        #[derive(Getter)]
        pub struct Test {
            code: i32,
        }
        let test = Test { code: 4 };
        assert_eq!(test.get_code(), &4);
    }
}
