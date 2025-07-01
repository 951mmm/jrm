use std::{collections::HashMap, fmt::Debug, ops::Range};

use jrm_macro::{ClassParser, KlassDebug, generate_ux};

use crate::{
    attribute::Attribute,
    class_reader::ClassReader,
    constant_pool::{ConstantClass, ConstantPool, ConstantWrapper},
};

pub struct ParserContext {
    pub count: usize,
    pub constant_index_range: Range<u16>,
    pub constant_pool: Vec<ConstantWrapper>,
}

impl ParserContext {
    pub fn new() -> Self {
        Self {
            count: 0,
            constant_index_range: 0..0,
            constant_pool: vec![],
        }
    }
}
pub trait ClassParser {
    fn parse(class_reader: &mut ClassReader, ctx: &mut ParserContext) -> anyhow::Result<Self>
    where
        Self: Sized;
}

generate_ux! {}
impl ClassParser for i32 {
    fn parse(class_reader: &mut ClassReader, _: &mut ParserContext) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let num = class_reader.read_four_bytes().unwrap_or(0);
        Ok(num as i32)
    }
}
impl ClassParser for f32 {
    fn parse(class_reader: &mut ClassReader, _: &mut ParserContext) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let num = class_reader.read_four_bytes().unwrap_or(0);
        Ok(f32::from_bits(num))
    }
}
impl ClassParser for i64 {
    fn parse(class_reader: &mut ClassReader, _: &mut ParserContext) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let high = class_reader.read_four_bytes().unwrap_or(0) as u64;
        let low = class_reader.read_four_bytes().unwrap_or(0) as u64;
        let num = (high << 32) | low;
        Ok(num as i64)
    }
}
impl ClassParser for f64 {
    fn parse(class_reader: &mut ClassReader, _: &mut ParserContext) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let high = class_reader.read_four_bytes().unwrap_or(0) as u64;
        let low = class_reader.read_four_bytes().unwrap_or(0) as u64;
        let num = (high << 32) | low;
        Ok(f64::from_bits(num))
    }
}

#[derive(KlassDebug, ClassParser)]
pub struct InstanceKlass {
    #[hex]
    magic: u32,
    minor_version: u16,
    major_version: u16,
    #[set_count]
    #[constant_index_end]
    constant_pool_count: u16,
    constant_pool: ConstantPool,
    #[hex]
    #[constant_index_check]
    access_flags: u16,
    #[constant_index_check]
    this_class: u16,
    #[constant_index_check]
    super_class: u16,
    #[set_count]
    interfaces_count: u16,
    #[impl_sized]
    interfaces: Vec<Interface>,
    #[set_count]
    fields_count: u16,
    #[impl_sized]
    fields: Vec<Field>,
    #[set_count]
    methods_count: u16,
    #[impl_sized]
    methods: Vec<Method>,
    // attributes_count: u16,
    // attributes: Vec<AttributeInfo>,
}
type Interface = ConstantClass;
#[derive(Debug, ClassParser)]
pub struct Field(Property);

#[derive(Debug, ClassParser)]
pub struct Method(Property);

#[derive(Debug, ClassParser)]
pub struct Property {
    access_flags: u16,
    name_index: u16,
    descriptor_index: u16,
    #[set_count]
    attributes_count: u16,
    // #[impl_sized(attributes_count)]
    // attributes: Vec<Attribute>,
}
