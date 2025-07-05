use std::{any::Any, collections::HashMap, fmt::Debug, ops::Range};

use jrm_macro::{ClassParser, KlassDebug, generate_ux, impl_class_parser_for_vec};
use maplit::hashmap;

use crate::{
    attributes::Attribute,
    class_reader::ClassReader,
    constant_pool::{ConstantClass, ConstantPool},
};

pub trait ContextIndex {
    type Idx;
    fn get(&self, index: Self::Idx) -> anyhow::Result<String>;
}

impl ContextIndex for HashMap<u8, &'static str> {
    type Idx = u8;
    fn get(&self, index: Self::Idx) -> anyhow::Result<String> {
        Ok(self[&index].to_owned())
    }
}
pub struct ParserContext {
    pub count: usize,
    pub constant_index_range: Range<u16>,
    pub constant_pool: ConstantPool,
    pub constant_tag_map: HashMap<u8, &'static str>,
    pub enum_entry: Box<dyn Any>,
}

impl ParserContext {
    pub fn new() -> Self {
        let constant_tag_map = hashmap! {
            0 => "Invalid",
            1 => "Utf8",
            3 => "Integer",
            4 => "Float",
            5 => "Long",
            6 => "Double",
            7 => "Class",
            8 => "String",
            9 => "FieldRef",
            10 => "MethodRef",
            11 => "InterfaceMethodRef",
            12 => "NameAndType",
            15 => "MethodHandle",
            16 => "MethodType",
            17 => "DynamicType",
            18 => "InvokeDynamic",
            19 => "Module",
            20 => "Package",
        };
        Self {
            count: Default::default(),
            constant_index_range: Default::default(),
            constant_pool: Default::default(),
            constant_tag_map,
            enum_entry: Box::new(i32::default()),
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
impl_class_parser_for_vec! {u8}

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
    constant_pool: ConstantPool,
    #[hex]
    #[constant_index(check)]
    access_flags: u16,
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
    // #[count(set)]
    // attributes_count: u16,
    // #[count(get)]
    // attributes: Vec<Attribute>,
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
    #[count(set)]
    attributes_count: u16,
    attributes: Vec<Attribute>,
}
