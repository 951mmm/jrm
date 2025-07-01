use std::{collections::HashMap, fmt::Debug};

use jrm_macro::{ClassParser, KlassDebug, generate_ux};

use crate::{attribute::Attribute, class_reader::ClassReader, constant_pool::ConstantPool};

#[derive(Clone, Debug)]
pub enum StoreType {
    Usize(usize),
    ConstantPool(ConstantPool),
}
impl From<usize> for StoreType {
    fn from(value: usize) -> Self {
        StoreType::Usize(value)
    }
}

impl From<ConstantPool> for StoreType {
    fn from(value: ConstantPool) -> Self {
        StoreType::ConstantPool(value)
    }
}
pub struct ParserContext {
    pub store: HashMap<String, StoreType>,
}

impl ParserContext {
    pub fn new() -> Self {
        let store = HashMap::new();
        Self { store }
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
    #[set_ctx]
    constant_pool_count: u16,
    constant_pool: ConstantPool,
    #[hex]
    access_flags: u16,
    this_class: u16,
    super_class: u16,
    #[set_ctx]
    interfaces_count: u16,
    #[impl_sized(constant_pool_count)]
    interfaces: Vec<u16>,
    #[set_ctx]
    fields_count: u16,
    #[impl_sized(fields_count)]
    fields: Vec<Field>,
    #[set_ctx]
    methods_count: u16,
    #[impl_sized(methods_count)]
    methods: Vec<Method>,
    // attributes_count: u16,
    // attributes: Vec<AttributeInfo>,
}

#[derive(Debug, ClassParser)]
pub struct Field(Property);

#[derive(Debug, ClassParser)]
pub struct Method(Property);

#[derive(Debug, ClassParser)]
pub struct Property {
    access_flags: u16,
    name_index: u16,
    descriptor_index: u16,
    #[set_ctx]
    attributes_count: u16,
    #[impl_sized(attributes_count)]
    attributes: Vec<Attribute>,
}
