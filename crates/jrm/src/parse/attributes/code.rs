use crate::parse::class_file_parser::{ClassParser, ParserContext};
use jrm_macro::{ClassParser, base_attribute};

use super::Attribute;

#[base_attribute(suffix(count_ident = attributes_count, item_ty = Attribute), impled)]
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

#[base_attribute(suffix(count_ident = line_number_table_length, item_ty = LineNumber, rename = line_number_table))]
#[derive(Debug, ClassParser)]
pub struct LineNumberTableAttribute {}
#[derive(Debug, ClassParser)]
pub struct LineNumber {
    pub start_pc: u16,
    pub line_number: u16,
}

#[base_attribute(suffix(
    count_ident = local_variable_table_length,
    item_ty = LocalVariable,
    rename = local_variable_table
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

// #[base_attribute(suffix(number_of_entries, StackMapFrame, rename(entries)))]
// #[derive(Debug, ClassParser)]
// pub struct StackMapTableAttribute {}

// #[derive(Debug, StackMapFrame)]
