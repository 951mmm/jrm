use quote::quote;
use syn::{Field, Fields, ItemStruct, Token, parse::Parse, parse_quote, punctuated::Punctuated};
pub struct Ast {
    pub structs: Vec<ItemStruct>,
}
impl Parse for Ast {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut structs = vec![];
        while !input.is_empty() {
            let item: ItemStruct = input.parse()?;
            structs.push(item);
        }
        Ok(Self { structs })
    }
}
pub fn define_constants_inner(structs: &mut Vec<ItemStruct>) -> proc_macro2::TokenStream {
    let prefix: Field = parse_quote!(
        #[enum_entry(get)]
        pub tag: u8
    );
    for item_struct in structs.iter_mut() {
        item_struct.attrs.push(parse_quote!(
            #[derive(Clone, Debug, ClassParser)]
        ));
        if let Fields::Named(ref mut fields_named) = item_struct.fields {
            let mut new_named: Punctuated<Field, Token![,]> = Punctuated::new();
            new_named.push(prefix.clone());
            new_named.extend(fields_named.named.clone());
            fields_named.named = new_named;
        }
    }

    quote! {
        #(#structs)*
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{define_constants, define_constants_inner};
    use macro_utils::print_expanded_fmt;
    use syn::parse_str;

    #[test]
    fn test_ast_parse() -> Result<(), Box<dyn Error>> {
        let code = r#"
            struct Some {
                A: i32,
            }
            struct Some2;
        "#;
        let ast: define_constants::Ast = parse_str(code)?;
        assert_eq!(ast.structs.len(), 2);
        Ok(())
    }
    #[test]
    fn test_define_constants_expand() -> Result<(), Box<dyn Error>> {
        let code = r#"
            struct Some {
                A: i32
            }
            struct Some2 {
                B: u8,
                C: u16,
            }
        "#;
        let mut ast: define_constants::Ast = parse_str(code)?;
        let expanded = define_constants_inner(&mut ast.structs);
        let raw_code = expanded.to_string();
        assert!(raw_code.contains("pub tag : u8 , A : i32"));
        assert!(raw_code.contains("pub tag : u8 , B : u8"));
        assert!(raw_code.contains("# [derive (Clone , Debug , ClassParser)]"));
        print_expanded_fmt(expanded);
        Ok(())
    }
}
