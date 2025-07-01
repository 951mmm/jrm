use crate::class_file_parser::{ClassParser, ParserContext};
use crate::class_reader::ClassReader;
use jrm_macro::ClassParser;

#[derive(Debug, ClassParser)]
pub struct Attribute {
    attribute_name_index: u16,
    #[set_count]
    attribute_length: u32,
    #[impl_sized(attribute_length)]
    info: Vec<AttributeInfo>,
}
#[derive(Debug, ClassParser)]
pub struct AttributeInfo {
    some: u8,
}
