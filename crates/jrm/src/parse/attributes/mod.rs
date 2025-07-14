mod code;

use crate::parse::class_file_parser::{ClassParser, ContextIndex, ParserContext};
use jrm_macro::{ClassParser, attribute_enum, base_attribute, impl_class_parser_for_vec};

use code::*;
attribute_enum! {SourceFile, Code, LineNumberTable, LocalVariableTable}
impl_class_parser_for_vec! {Attribute}

#[base_attribute(single(ident = sourcefile_index, ty = u16, constant_index_check))]
#[derive(Debug, ClassParser)]
pub struct SourceFileAttribute {}
