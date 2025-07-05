use quote::quote;
use syn::{Fields, Ident, ItemStruct, parse::Parse, parse_quote};

#[derive(Default)]
pub struct Attr {
    one_word: bool,
    two_words: bool,
    __ref: bool,
    dynamic: bool,
    module: bool,
}
impl Parse for Attr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut attr: Attr = Default::default();
        let ident: Ident = input.parse()?;
        match ident.to_string().as_str() {
            "one_word" => attr.one_word = true,
            "two_words" => attr.two_words = true,
            "__ref" => attr.__ref = true,
            "dynamic" => attr.dynamic = true,
            "module" => attr.module = true,
            _ => {}
        }
        if !input.is_empty() {
            return Err(input.error("invalid attr count"));
        }
        Ok(attr)
    }
}
pub fn constant_inner(attr: &Attr, item_struct: &mut ItemStruct) -> proc_macro2::TokenStream {
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
        if attr.one_word {
            fields_named.named.push(one_word_field);
        }
        if attr.two_words {
            for two_words_field in two_words_fields {
                fields_named.named.push(two_words_field);
            }
        }
        if attr.__ref {
            for ref_field in ref_fields {
                fields_named.named.push(ref_field);
            }
        }
        if attr.dynamic {
            for dynamic_field in dynamic_fields {
                fields_named.named.push(dynamic_field);
            }
        }
        if attr.module {
            fields_named.named.push(module_field);
        }
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
        let attr_module: constant::Attr = parse_quote!(module);
        let mut __struct = generate_struct();
        let expanded = constant_inner(&attr_module, &mut __struct);
        let raw_code = expanded.to_string();
        assert!(raw_code.contains("pub name_index : u16"));
        print_expanded_fmt(expanded);
    }
    fn generate_struct() -> ItemStruct {
        parse_quote!(
            struct TestStruct {}
        )
    }
    fn expand_attr(attr: constant::Attr) -> (String, proc_macro2::TokenStream) {
        let mut __struct = generate_struct();
        let expanded = constant_inner(&attr, &mut __struct);
        let raw_code = expanded.to_string();
        (raw_code, expanded)
    }
    #[test]
    #[should_panic(expected = "invalid attr count")]
    fn test_constant_attr_count_err() {
        let _attrs: constant::Attr = parse_quote!(module, __ref);
    }
}
