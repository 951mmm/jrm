
use macro_utils::syn_err;
use quote::quote;
use syn::{
    Fields, Ident, ItemStruct, Token, parse::Parse, parse_quote,
};

pub struct Attrs {
    constant_feature: ConstantFeature,
}
pub enum ConstantFeature {
    OneWord,
    TwoWord,
    Ref,
    Dynamic,
    Module,
}

impl Parse for ConstantFeature {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        let result = match ident.to_string().as_str() {
            "one_word" => Self::OneWord,
            "two_words" => Self::TwoWord,
            "__ref" => Self::Ref,
            "dynamic" => Self::Dynamic,
            "module" => Self::Module,
            _ => {
                syn_err!("invalid constant feature: {}", ident);
            }
        };
        Ok(result)
    }
}

impl Parse for Attrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        if ident == "feature" {
            input.parse::<Token![=]>()?;
            let constant_feature: ConstantFeature = input.parse()?;
            return Ok(Attrs { constant_feature });
        }
        syn_err!("invalid attrs, expected `feature = ...`");
    }
}
pub fn constant_inner(attr: &Attrs, item_struct: &mut ItemStruct) -> proc_macro2::TokenStream {
    let one_word_field = parse_quote!(pub bytes: u32);
    let two_words_fields = [
        parse_quote!(pub high_bytes: u32),
        parse_quote!(pub low_bytes: u32),
    ];
    let ref_fields = [
        parse_quote!(pub class_index: u16),
        parse_quote!(pub name_and_type_index: u16),
    ];
    let dynamic_fields = [
        parse_quote!(pub bootstrap_method_attr_index: u16),
        parse_quote!(pub name_and_type_index: u16),
    ];
    let module_field = parse_quote!(pub name_index: u16);
    if let Fields::Named(ref mut fields_named) = item_struct.fields {
        let Attrs { constant_feature } = attr;
        match constant_feature {
            ConstantFeature::OneWord => {
                fields_named.named.push(one_word_field);
            }
            ConstantFeature::TwoWord => {
                for two_words_field in two_words_fields {
                    fields_named.named.push(two_words_field);
                }
            }
            ConstantFeature::Ref => {
                for ref_field in ref_fields {
                    fields_named.named.push(ref_field);
                }
            }
            ConstantFeature::Dynamic => {
                for dynamic_field in dynamic_fields {
                    fields_named.named.push(dynamic_field);
                }
            }
            ConstantFeature::Module => {
                fields_named.named.push(module_field);
            }
        };
    }
    quote! {
        #item_struct
    }
}
#[cfg(test)]
mod tests {

    use syn::{ItemStruct, parse_quote};

    use macro_utils::print_expanded_fmt;

    use crate::{constant, constant_inner};

    #[test]
    fn test_constant_one_word_expand() {
        let (raw_code, expanded) = expand_attr(parse_quote!(one_word));
        assert!(raw_code.contains("pub bytes : u32"));
        print_expanded_fmt(expanded);
    }
    #[test]
    fn test_constant_two_word_expand() {
        let (raw_code, expanded) = expand_attr(parse_quote!(two_words));
        assert!(raw_code.contains("high_bytes"));
        assert!(raw_code.contains("pub low_bytes : u32"));
        print_expanded_fmt(expanded);
    }
    #[test]
    fn test_constant_attr_ref_expand() {
        let (raw_code, expanded) = expand_attr(parse_quote!(__ref));
        assert!(raw_code.contains("class_index"));
        assert!(raw_code.contains("pub name_and_type_index : u16"));
        print_expanded_fmt(expanded);
    }
    #[test]
    fn test_constant_attr_dynamic_expand() {
        let (raw_code, expanded) = expand_attr(parse_quote!(dynamic));
        assert!(raw_code.contains("name_and_type_index"));
        assert!(raw_code.contains("pub bootstrap_method_attr_index : u16"));
        print_expanded_fmt(expanded);
    }
    #[test]
    fn test_constant_attr_module_expand() {
        let (raw_code, expanded) = expand_attr(parse_quote!(module));
        assert!(raw_code.contains("pub name_index : u16"));
        print_expanded_fmt(expanded);
    }
    fn generate_struct() -> ItemStruct {
        parse_quote!(
            struct TestStruct {}
        )
    }
    fn expand_attr(attr: constant::Attrs) -> (String, proc_macro2::TokenStream) {
        let mut __struct = generate_struct();
        let expanded = constant_inner(&attr, &mut __struct);
        let raw_code = expanded.to_string();
        (raw_code, expanded)
    }
    #[test]
    #[should_panic(expected = "invalid attr count")]
    fn test_constant_attr_count_err() {
        let _attrs: constant::Attrs = parse_quote!(module, __ref);
    }
}

