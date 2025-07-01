use crate::class_file_parser::{ClassLookUpParser, ClassParser};
use crate::class_reader::ClassReader;
use jrm_macro::ClassParser;

#[derive(Debug, ClassParser)]
pub struct Attribute {
    attribute_name_index: u16,
    attribute_length: u32,
    #[with_lookup(attribute_length)]
    #[impl_sized]
    info: Vec<AttributeInfo>,
}
#[derive(Debug, ClassParser)]
pub struct AttributeInfo {
    some: u8,
}
