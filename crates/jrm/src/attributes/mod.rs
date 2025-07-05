mod code;

use crate::class_file_parser::{ClassParser, ContextIndex, ParserContext};
use crate::class_reader::ClassReader;
use jrm_macro::{ClassParser, attribute_enum, base_attribute, impl_class_parser_for_vec};

use code::Exception;

attribute_enum! {Code, LineNumberTable}
impl_class_parser_for_vec! {Attribute}

// FIXME 注意宏的作用顺序
#[base_attribute(suffix(attributes_count, Attribute))]
#[derive(Debug, ClassParser)]
pub struct CodeAttribute {
    pub max_stack: u16,
    pub max_locals: u16,
    #[count(set)]
    pub code_length: u32,
    #[count(get_bytes)]
    pub code: Vec<u8>,
    #[count(set)]
    pub exception_table_length: u16,
    #[count(get)]
    pub exception_table: Vec<Exception>,
}

#[base_attribute(suffix(line_number_table_length, LineNumberTable))]
#[derive(Debug, ClassParser)]
pub struct LineNumberTableAttribute {}
#[derive(Debug, ClassParser)]
pub struct LineNumberTable {}
