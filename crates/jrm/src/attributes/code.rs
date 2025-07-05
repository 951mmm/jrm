use crate::class_file_parser::{ClassParser, ParserContext};
use crate::class_reader::ClassReader;
use jrm_macro::ClassParser;

#[derive(Debug, ClassParser)]
pub struct Exception {
    start_pc: u16,
    end_pc: u16,
    handler_pc: u16,
    catch_type: u16,
}
