use quote::quote;
use syn::{Ident, Lit, Token, parse::Parse};

pub struct Args {
    pub ident: Option<Ident>,
    pub msg: Lit,
}

impl Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = if input.peek(Ident) {
            let ident: Ident = input.parse()?;
            input.parse::<Token![,]>()?;
            Some(ident)
        } else {
            None
        };
        let msg: Lit = input.parse()?;
        Ok(Self { ident, msg })
    }
}
pub fn syn_err_inner(ast: &Args) -> proc_macro2::TokenStream {
    let Args { ident, msg } = ast;
    match ident {
        Some(ident) => {
            quote! {
                return Err(syn::Error::new_spanned(#ident, #msg))
            }
        }
        None => {
            quote! {
                return Err(syn::Error::new(proc_macro2::Span::mixed_site(), #msg))
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use syn::parse_quote;

    use crate::{syn_err::Args, syn_err_inner};

    #[test]
    fn test_syn_err_2_param_expand() {
        let ast: Args = parse_quote!(some, "some parse err");
        let expanded = syn_err_inner(&ast);
        let raw_code = expanded.to_string();
        assert!(raw_code.contains("some ,"));
        println!("{}", expanded);
    }
    #[test]
    fn test_syn_err_1_param_expand() {
        let ast: Args = parse_quote!("some parse err");
        let expanded = syn_err_inner(&ast);
        let raw_code = expanded.to_string();
        assert!(raw_code.contains("Span :: mixed_site ()"));
        println!("{}", expanded);
    }
}
