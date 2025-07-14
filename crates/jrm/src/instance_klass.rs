use std::sync::Arc;

use bitflags::{Flags, bitflags};

use crate::attributes::Attribute;
use crate::class_file_parser::{ClassParser, ParserContext};
use crate::constant_pool::{ConstantClass, ConstantPool};
use crate::runtime::Method as FrameMethod;
use jrm_macro::{ClassParser, KlassDebug};

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
    #[derive(Debug, Clone, Copy)]
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

#[derive(KlassDebug, ClassParser)]
pub struct InstanceKlass {
    #[hex]
    magic: u32,
    minor_version: u16,
    major_version: u16,
    #[count(set)]
    #[constant_index(setend)]
    constant_pool_count: u16,
    #[constant_pool(set)]
    constant_pool: Arc<ConstantPool>,
    #[hex]
    access_flags: ClassAccessFlags,
    #[constant_index(check)]
    this_class: u16,
    #[constant_index(check)]
    super_class: u16,
    #[count(set)]
    interfaces_count: u16,
    #[count(get)]
    interfaces: Vec<Interface>,
    #[count(set)]
    fields_count: u16,
    #[count(get)]
    fields: Vec<Field>,
    #[count(set)]
    methods_count: u16,
    #[count(get)]
    methods: Vec<Method>,
    #[count(set)]
    attributes_count: u16,
    #[count(impled)]
    attributes: Vec<Attribute>,
}
impl InstanceKlass {
    pub fn find_method(&self, name: &str, descriptor: &str) -> FrameMethod {
        let mut max_locals = None;
        let mut max_stack = None;
        let mut code = None;
        let mut is_static = false;
        unsafe {
            self.methods
                .iter()
                .find(|method| {
                    let method_name = self.constant_pool.get_utf8_string(method.name_index);
                    let method_descriptor =
                        self.constant_pool.get_utf8_string(method.descriptor_index);
                    method_name == name || method_descriptor == descriptor
                })
                .map(|method| {
                    is_static = method.access_flags.contains(MethodAccessFlags::STATIC);
                    &method.attributes
                })
                .unwrap_unchecked()
                .iter()
                .for_each(|attr| {
                    if let Attribute::Code(code_attr) = attr {
                        max_locals = Some(code_attr.max_locals);
                        max_stack = Some(code_attr.max_stack);
                        code = Some(&code_attr.code);
                    }
                });

            FrameMethod {
                name: name.to_string(),
                descriptor: descriptor.to_string(),
                max_locals: max_locals.unwrap_unchecked(),
                max_stack: max_stack.unwrap_unchecked(),
                code: code.unwrap_unchecked().clone(),
                is_static,
            }
        }
    }
    pub fn get_constant_pool(&self) -> Arc<ConstantPool> {
        self.constant_pool.clone()
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

#[derive(Debug, ClassParser)]
pub struct Method {
    access_flags: MethodAccessFlags,
    name_index: u16,
    descriptor_index: u16,
    #[count(set)]
    attributes_count: u16,
    #[count(impled)]
    attributes: Vec<Attribute>,
}

#[cfg(test)]
mod tests {
    use crate::{instance_klass::ClassAccessFlags, test_context::TestContext};

    #[test]
    fn test_class_access_flag() {
        let instance_klass = TestContext::parse_class_file("Simple1Impl.class");
        assert!(
            !instance_klass
                .access_flags
                .contains(ClassAccessFlags::PUBLIC)
        )
    }

    #[test]
    fn test_find_method() {
        let instance_klass = TestContext::parse_class_file("Simple1Impl.class");
        let method = instance_klass.find_method("main", "([Ljava/lang/String)V");
        println!("method {} is: {:?}", method.name, method);
        assert!(method.is_static);
    }
}
