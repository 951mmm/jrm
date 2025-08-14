use jrm_macro::{ClassParser, Getter, ParseVariant, base_attribute};

use crate::class_file_parser::{ClassParser, ContextIndex, ParserContext};

#[base_attribute(suffix(count_ident=num_annotations, item_ty=Annotation))]
#[derive(Debug, ClassParser, Getter)]
pub struct RuntimeVisibleAnnotationsAttribute {}

#[derive(Debug, ClassParser, Getter)]
pub struct Annotation {
    #[class_parser(constant_index(check))]
    #[getter(copy)]
    type_index: u16,

    #[class_parser(count(set))]
    #[getter(skip)]
    num_element_value_pairs: u16,

    #[class_parser(count(get))]
    element_value_pairs: Vec<ElementValuePair>,
}

#[derive(Debug, ClassParser)]
pub struct ElementValuePair {
    #[class_parser(constant_index(check))]
    element_name_index: u16,
    element_value: ElementValue,
}

#[derive(Debug, ClassParser)]
pub struct ElementValue {
    #[class_parser(enum_entry(set))]
    tag: u8,
    value: Value,
}

#[derive(Debug, ClassParser, ParseVariant)]
#[class_parser(enum_entry(index(map = element_value_map[u8], outer)))]
pub enum Value {
    ConstValueIndex(#[class_parser(constant_index(check))] u16),
    #[parse_variant(pass {
        #[derive(Debug, Getter)]
    })]
    EnumConstValue {
        #[class_parser(constant_index(check))]
        #[parse_variant(pass {
            #[getter(copy)]
        })]
        type_name_index: u16,
        #[class_parser(constant_index(check))]
        #[parse_variant(pass {
            #[getter(copy)]
        })]
        const_name_index: u16,
    },
    ClassInfoIndex(u16),
    AnnotationValue(Annotation),
    #[parse_variant(pass {
        #[derive(Debug, Getter)]
    })]
    ArrayValue {
        #[class_parser(count(set))]
        #[parse_variant(pass {
            #[getter(copy)]
        })]
        name_values: u16,
        #[class_parser(count(get))]
        #[parse_variant(pass {
            #[getter(copy)]
        })]
        values: Vec<ElementValue>,
    },
}
