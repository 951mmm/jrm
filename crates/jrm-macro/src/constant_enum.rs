use quote::{format_ident, quote};

use crate::build_enum_input;

pub fn constant_enum_inner(ast: &build_enum_input::Ast) -> proc_macro2::TokenStream {
    let variants = ast.idents.iter().map(|ident| {
        let constant_ident = format_ident!("Constant{}", ident);
        quote! {#ident(#constant_ident)}
    });

    quote! {
        #[derive(Clone, Debug, ClassParser, ParseVariant)]
        #[class_parser(enum_entry(index(map = constant_tag_map[u8])))]
        pub enum Constant {
            #(#variants,)*
            Invalid
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use macro_utils::print_expanded_fmt;

    use crate::{build_enum_input, constant_enum_inner};

    #[test]
    fn test_constant_enum_expanded() {
        let input: build_enum_input::Ast = parse_quote!(A, B, C);
        let expanded = constant_enum_inner(&input);
        let raw_code = expanded.to_string();
        assert!(raw_code.contains("pub enum Constant"));
        assert!(raw_code.contains("A (ConstantA)"));
        assert!(raw_code.contains("Result < & ConstantA >"));
        print_expanded_fmt(expanded);
    }
}
