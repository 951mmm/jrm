use std::fmt::Debug;

use jrm_macro::{ClassParser, KlassDebug, generate_u_parse};

use crate::{attribute::Attribute, class_reader::ClassReader, constant_pool::ConstantPool};

pub trait ClassParser {
    fn parse(class_reader: &mut ClassReader) -> anyhow::Result<Self>
    where
        Self: Sized;
}

pub trait ClassLookUpParser {
    fn parse(class_reader: &mut ClassReader, prev: usize) -> anyhow::Result<Self>
    where
        Self: Sized;
}
generate_u_parse! {}
impl ClassParser for i32 {
    fn parse(class_reader: &mut ClassReader) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let num = class_reader.read_four_bytes().unwrap_or(0);
        Ok(num as i32)
    }
}
impl ClassParser for f32 {
    fn parse(class_reader: &mut ClassReader) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let num = class_reader.read_four_bytes().unwrap_or(0);
        Ok(f32::from_bits(num))
    }
}
impl ClassParser for i64 {
    fn parse(class_reader: &mut ClassReader) -> anyhow::Result<Self>
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
    fn parse(class_reader: &mut ClassReader) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let high = class_reader.read_four_bytes().unwrap_or(0) as u64;
        let low = class_reader.read_four_bytes().unwrap_or(0) as u64;
        let num = (high << 32) | low;
        Ok(f64::from_bits(num))
    }
}
impl ClassLookUpParser for Vec<u8> {
    fn parse(class_reader: &mut ClassReader, prev: usize) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(class_reader.read_bydes(prev).unwrap_or_default())
    }
}

#[derive(KlassDebug, ClassParser)]
pub struct InstanceKlass {
    #[hex]
    magic: u32,
    minor_version: u16,
    major_version: u16,
    constant_pool_count: u16,
    #[with_lookup(constant_pool_count)]
    constant_pool: ConstantPool,
    #[hex]
    access_flags: u16,
    this_class: u16,
    super_class: u16,
    interfaces_count: u16,
    #[with_lookup(interfaces_count)]
    #[impl_sized]
    interfaces: Vec<u16>,
    fields_count: u16,
    #[with_lookup(fields_count)]
    #[impl_sized]
    fields: Vec<Field>,
    methods_count: u16,
    #[with_lookup(methods_count)]
    #[impl_sized]
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
    attributes_count: u16,
    #[with_lookup(attributes_count)]
    #[impl_sized]
    attributes: Vec<Attribute>,
}
