use crate::class_file_parser::{ClassParser, ContextIndex, ParserContext};
use crate::class_reader::ClassReader;
use jrm_macro::{ClassParser, attribute_enum, base_attribute, impl_class_parser_for_vec};

attribute_enum! {Code, LineNumberTable, LocalVariableTable}
impl_class_parser_for_vec! {Attribute}

// FIXME 注意宏的作用顺序
#[base_attribute(suffix(attributes_count, Attribute), impled)]
#[derive(Debug, ClassParser)]
pub struct CodeAttribute {
    pub max_stack: u16,
    pub max_locals: u16,
    #[count(set)]
    pub code_length: u32,
    #[count(impled)]
    pub code: Vec<u8>,
    #[count(set)]
    pub exception_table_length: u16,
    #[count(get)]
    pub exception_table: Vec<Exception>,
}
#[derive(Debug, ClassParser)]
pub struct Exception {
    start_pc: u16,
    end_pc: u16,
    handler_pc: u16,
    catch_type: u16,
}

#[base_attribute(suffix(line_number_table_length, LineNumber, rename(line_number_table)))]
#[derive(Debug, ClassParser)]
pub struct LineNumberTableAttribute {}
#[derive(Debug, ClassParser)]
pub struct LineNumber {
    pub start_pc: u16,
    pub line_number: u16,
}

#[base_attribute(suffix(
    local_variable_table_length,
    LocalVariable,
    rename(lcoal_variable_table)
))]
#[derive(Debug, ClassParser)]
pub struct LocalVariableTableAttribute {}
#[derive(Debug, ClassParser)]
pub struct LocalVariable {
    pub start_pc: u16,
    pub length: u16,
    pub name_index: u16,
    pub description_index: u16,
    pub index: u16,
}
