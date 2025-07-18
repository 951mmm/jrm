use std::{
    any::Any,
    collections::HashMap,
    ops::{Deref, Range},
    sync::Arc,
};

use jrm_macro::{generate_ux, impl_class_parser_for_vec};
use maplit::hashmap;

use crate::{
    class_reader::ClassReader,
    constant_pool::ConstantPool,
};

pub trait ContextIndex {
    type Idx;
    fn get(&self, index: Self::Idx) -> String;
}

/// TODO 同129行
impl<T: ContextIndex> ContextIndex for Arc<T> {
    type Idx = T::Idx;
    fn get(&self, index: Self::Idx) -> String {
        self.deref().get(index)
    }
}

impl ContextIndex for HashMap<u8, &'static str> {
    type Idx = u8;
    fn get(&self, index: Self::Idx) -> String {
        self[&index].to_string()
    }
}
pub struct ParserContext {
    pub class_reader: ClassReader,
    pub count: usize,
    pub constant_index_range: Range<u16>,
    pub constant_pool: Arc<ConstantPool>,
    pub constant_tag_map: HashMap<u8, &'static str>,
    pub enum_entry: Box<dyn Any>,
}

impl From<Vec<u8>> for ParserContext {
    fn from(value: Vec<u8>) -> Self {
        let class_reader = value.into();
        Self::new(class_reader)
    }
}

impl ParserContext {
    pub fn new(class_reader: ClassReader) -> Self {
        // TODO parser context的静态化
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
            class_reader,
            count: Default::default(),
            constant_index_range: Default::default(),
            constant_pool: Default::default(),
            constant_tag_map,
            enum_entry: Box::new(i32::default()),
        }
    }
}

pub trait ClassParser {
    fn parse(ctx: &mut ParserContext) -> anyhow::Result<Self>
    where
        Self: Sized;
}

generate_ux! {}

impl ClassParser for i32 {
    fn parse(ctx: &mut ParserContext) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let num = ctx.class_reader.read_four_bytes().unwrap_or(0);
        Ok(num as i32)
    }
}
impl ClassParser for f32 {
    fn parse(ctx: &mut ParserContext) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let num = ctx.class_reader.read_four_bytes().unwrap_or(0);
        Ok(f32::from_bits(num))
    }
}
impl ClassParser for i64 {
    fn parse(ctx: &mut ParserContext) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let high = ctx.class_reader.read_four_bytes().unwrap_or(0) as u64;
        let low = ctx.class_reader.read_four_bytes().unwrap_or(0) as u64;
        let num = (high << 32) | low;
        Ok(num as i64)
    }
}
impl ClassParser for f64 {
    fn parse(ctx: &mut ParserContext) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let high = ctx.class_reader.read_four_bytes().unwrap_or(0) as u64;
        let low = ctx.class_reader.read_four_bytes().unwrap_or(0) as u64;
        let num = (high << 32) | low;
        Ok(f64::from_bits(num))
    }
}
impl_class_parser_for_vec! {u8}

//TODO 可以写进宏，但是会和集合混淆。需要生成宏
impl<T: ClassParser> ClassParser for Arc<T> {
    fn parse(ctx: &mut ParserContext) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let inner = <T as ClassParser>::parse(ctx)?;
        Ok(Arc::new(inner))
    }
}
