use convert_case::{Case, Casing};
use quote::{format_ident, quote};
use syn::{Ident, Token, parse::Parse, punctuated::Punctuated};

pub struct Ast {
    pub idents: Punctuated<Ident, Token![,]>,
}
impl Parse for Ast {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let idents = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;
        Ok(Ast { idents })
    }
}

/// 为enum实现parse cast函数
/// 每个parse函数返回[anyhow::Result]
#[deprecated]
pub fn generate_parse_cast_impl(
    enum_ident_lit: &str,
    idents: &Punctuated<Ident, Token![,]>,
    varent_ident_after_enum_ident: bool,
) -> proc_macro2::TokenStream {
    let enum_ident = format_ident!("{}", enum_ident_lit);
    let parse_cast_fns = idents.iter().map(|ident| {
        let fn_ident = format_ident!(
            "parse_{}",
            ident
                .to_string()
                .from_case(Case::Camel)
                .to_case(Case::Snake)
        );
        let variant_ident = if varent_ident_after_enum_ident {
            format_ident!("{}{}", enum_ident_lit, ident)
        } else {
            format_ident!("{}{}", ident, enum_ident_lit)
        };
        let variant_var_ident = format_ident!(
            "{}",
            variant_ident
                .to_string()
                .from_case(Case::Camel)
                .to_case(Case::Snake)
        );
        let item_fn = quote! {
            #[inline]
            #[allow(dead_code)]
            pub fn #fn_ident(&self) -> anyhow::Result<&#variant_ident> {
                if let #enum_ident::#ident(#variant_var_ident) = self {
                    return Ok(&#variant_var_ident)
                }
                anyhow::bail!("failed to parse as {}", stringify!(#variant_ident))
            }
        };
        quote! {
            #item_fn
        }
    });
    quote! {
        impl #enum_ident {
            #(#parse_cast_fns)*
        }
    }
}
#[cfg(test)]
mod tests {
    use std::error::Error;

    use syn::parse_str;

    use crate::build_enum_input;

    #[test]
    fn test_ast_parse() -> Result<(), Box<dyn Error>> {
        let code = r#"
            A, B, C
        "#;
        let ast: build_enum_input::Ast = parse_str(code)?;
        assert_eq!(ast.idents.len(), 3);
        assert_eq!(ast.idents[1], "B");
        Ok(())
    }
}
