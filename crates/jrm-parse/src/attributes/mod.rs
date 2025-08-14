mod annotation;
mod code;

use crate::ConstantClass;
use crate::class_file_parser::{ClassParser, ContextIndex, ParserContext};
use jrm_macro::{
    ClassParser, Getter, ParseVariant, attribute_enum, base_attribute, impl_class_parser_for_vec,
};

use annotation::*;
pub use code::Exception;
use code::*;

attribute_enum! {
    SourceFile,
    // SourceDebugExtension

    // code
    Code, LineNumberTable, LocalVariableTable, // LocalVariableTypeTable

    Exception,
    Deprecated,

    // annotation
    RuntimeVisibleAnnotations

}
impl_class_parser_for_vec! {Attribute}

// TODO getter for single
#[base_attribute(single(ident = sourcefile_index, ty = u16, constant_index_check))]
#[derive(Debug, ClassParser, Getter)]
pub struct SourceFileAttribute {}

#[base_attribute(suffix(count_ident = number_of_exceptions, item_ty = ConstantClass, rename=expcetion_index_table), impled)]
#[derive(Debug, ClassParser, Getter)]
pub struct ExceptionAttribute {}

#[base_attribute]
#[derive(Debug, ClassParser, Getter)]
pub struct DeprecatedAttribute {}
