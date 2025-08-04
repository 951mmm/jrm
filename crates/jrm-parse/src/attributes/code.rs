use crate::class_file_parser::{ClassParser, ParserContext};
use jrm_macro::{ClassParser, Getter, base_attribute};

use super::Attribute;

#[base_attribute(suffix(count_ident = attributes_count, item_ty = Attribute), impled)]
#[derive(Debug, ClassParser, Getter)]
pub struct CodeAttribute {
    #[getter(copy)]
    max_stack: u16,

    #[getter(copy)]
    max_locals: u16,

    #[class_parser(count(set))]
    #[getter(skip)]
    code_length: u32,

    #[class_parser(count(impled))]
    code: Vec<u8>,

    #[class_parser(count(set))]
    #[getter(skip)]
    exception_table_length: u16,

    #[class_parser(count(get))]
    exception_table: Vec<Exception>,
}

#[derive(Debug, ClassParser, Getter)]
pub struct Exception {
    #[getter(copy)]
    start_pc: u16,
    #[getter(copy)]
    end_pc: u16,
    #[getter(copy)]
    handler_pc: u16,
    #[getter(copy)]
    catch_type: u16,
}

#[base_attribute(suffix(count_ident = line_number_table_length, item_ty = LineNumber, rename = line_number_table))]
#[derive(Debug, ClassParser, Getter)]
pub struct LineNumberTableAttribute {}
#[derive(Debug, ClassParser, Getter)]
pub struct LineNumber {
    #[getter(copy)]
    start_pc: u16,
    #[getter(copy)]
    line_number: u16,
}

#[base_attribute(suffix(
    count_ident = local_variable_table_length,
    item_ty = LocalVariable,
    rename = local_variable_table
))]
#[derive(Debug, ClassParser, Getter)]
pub struct LocalVariableTableAttribute {}

#[derive(Debug, ClassParser, Getter)]
pub struct LocalVariable {
    #[getter(copy)]
    pub start_pc: u16,
    #[getter(copy)]
    pub length: u16,
    #[getter(copy)]
    pub name_index: u16,
    #[getter(copy)]
    pub description_index: u16,
    #[getter(copy)]
    pub index: u16,
}

// #[base_attribute(suffix(number_of_entries, StackMapFrame, rename(entries)))]
// #[derive(Debug, ClassParser)]
// pub struct StackMapTableAttribute {}

// #[derive(Debug, StackMapFrame)]
