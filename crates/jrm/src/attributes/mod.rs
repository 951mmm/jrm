mod code;

use jrm_macro::base_attribute;

use code::{Code, Exception};
pub enum Attribute {
    Code(CodeAttribute),
}

#[base_attribute(suffix(attributes_count, Attribute))]
pub struct CodeAttribute {
    pub max_stack: u16,
    pub max_locals: u16,
    pub code_length: u32,
    pub code: Vec<Code>,
    pub exception_table_length: u16,
    pub exception_table: Vec<Exception>,
}
