use quote::{format_ident, quote};

use crate::{build_enum_input::Ast, generate_parse_cast_impl};

pub fn attribute_enum_inner(ast: &Ast) -> proc_macro2::TokenStream {
    let variants = ast.idents.iter().map(|ident| {
        let attribute_ident = format_ident!("{}Attribute", ident);
        quote! {
            #ident(#attribute_ident)
        }
    });

    let parse_cast_impl = generate_parse_cast_impl("Attribute", &ast.idents, false);

    quote! {
        #[derive(Debug, ClassParser)]
        #[enum_entry(index(constant_pool[u16]))]
        pub enum Attribute {
            #(#variants),*
        }
        #parse_cast_impl
    }
}

#[cfg(test)]
mod test {
    use macro_utils::print_expanded_fmt;
    use syn::parse_quote;

    use crate::attribute_enum::attribute_enum_inner;

    #[test]
    fn test_attrbute_enum_expand() {
        let ast = parse_quote!(Attr1, Attr2);
        let expanded = attribute_enum_inner(&ast);
        let raw_code = expanded.to_string();
        println!("{}", raw_code);
        assert!(raw_code.contains("Result < & Attr1Attribute >"));
        print_expanded_fmt(expanded);
    }
}
