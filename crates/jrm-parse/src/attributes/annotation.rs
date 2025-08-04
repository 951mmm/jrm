use jrm_macro::{ClassParser, Getter, base_attribute};

use crate::class_file_parser::{ClassParser, ParserContext};

#[base_attribute(suffix(count_ident=num_annotations, item_ty=Annotation))]
#[derive(Debug, ClassParser, Getter)]
pub struct RuntimeVisibleAnnotationsAttribute {}

#[derive(Debug, ClassParser)]
pub struct Annotation {
    #[constant_index(check)]
    type_index: u16,
    #[count(set)]
    num_element_value_pairs: u16,
    #[count(get)]
    element_value_pairs: Vec<ElementValuePair>,
}

#[derive(Debug, ClassParser)]
pub struct ElementValuePair {
    #[constant_index(check)]
    element_name_index: u16,
    element_value: ElementValue,
}

#[derive(Debug, ClassParser)]
pub struct ElementValue {}
