use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Data, DeriveInput, Expr, Ident, Lit, LitStr, Token, parse::Parse, parse_macro_input,
    parse_quote,
};

#[proc_macro]
pub fn simple_field_attr(input: TokenStream) -> TokenStream {
    let arg = parse_macro_input!(input as LitStr);
    let fn_ident = format_ident!("attr_{}", arg.value());
    quote! {
        fn #fn_ident(field: &syn::Field) -> bool {
            field.attrs.iter().any(|attr| attr.path().is_ident(#arg))
        }
    }
    .into()
}

#[proc_macro]
pub fn unwrap_err(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);
    quote! {
        match #expr {
            Ok(tokens) => tokens.into(),
            Err(err) => err.to_compile_error().into(),
        }
    }
    .into()
}
struct SynErr {
    pub ident: Option<Ident>,
    pub msg: Lit,
}

impl Parse for SynErr {
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
#[proc_macro]
pub fn syn_err(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as SynErr);
    syn_err_inner(&ast).into()
}
fn syn_err_inner(ast: &SynErr) -> proc_macro2::TokenStream {
    let SynErr { ident, msg } = ast;
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

#[proc_macro_attribute]
pub fn attr_enum(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);
    let enum_ident = &ast.ident;
    let enum_ident_string = enum_ident
        .to_string()
        .from_case(Case::Camel)
        .to_case(Case::Snake);

    let mut if_items = vec![];
    if let Data::Enum(ref mut data_enum) = ast.data {
        let variants = &data_enum.variants;
        let mut iter_variants = variants.iter();
        if let Some(first) = iter_variants.next() {
            let first_ident = &first.ident;
            let first_lit = first
                .ident
                .to_string()
                .from_case(Case::Camel)
                .to_case(Case::Snake);
            if_items.push(quote! {
                let result = if op == #first_lit {
                    #enum_ident::#first_ident
                }
            });
        }

        for variant in iter_variants {
            let variant_ident = &variant.ident;
            let variant_lit = variant
                .ident
                .to_string()
                .from_case(Case::Camel)
                .to_case(Case::Snake);
            if_items.push(quote! {
                else if op == #variant_lit {
                    #enum_ident::#variant_ident
                }
            });
        }

        if !if_items.is_empty() {
            if_items.push(quote! {
                else {
                    return Err(syn::Error::new_spanned(
                        attr,
                        format_args!("failed to parse attr `{}`", #enum_ident_string),
                    ));
                };
            });
        }

        data_enum.variants.push(parse_quote!(None));
    }
    ast.attrs.push(parse_quote!(#[derive(Eq, PartialEq)]));

    let fn_ident = format_ident!("attr_{}", enum_ident_string);
    quote! {
        #ast
        fn #fn_ident(field: &syn::Field) -> syn::Result<#enum_ident> {
            for attr in &field.attrs {
                if attr.path().is_ident(#enum_ident_string) {
                    let op: Ident = attr.parse_args()?;
                    #(#if_items)*
                    return Ok(result);
                }
            }
            Ok(#enum_ident::None)
        }
    }
    .into()
}

#[cfg(test)]
mod tests {

    use syn::parse_quote;

    use crate::{SynErr, syn_err_inner};

    #[test]
    fn test_syn_err_2_param_expand() {
        let ast: SynErr = parse_quote!(some, "some parse err");
        let expanded = syn_err_inner(&ast);
        let raw_code = expanded.to_string();
        assert!(raw_code.contains("some ,"));
        println!("{}", expanded);
    }
    #[test]
    fn test_syn_err_param_expand() {
        let ast: SynErr = parse_quote!("some parse err");
        let expanded = syn_err_inner(&ast);
        let raw_code = expanded.to_string();
        assert!(raw_code.contains("Span :: call_site"));
        println!("{}", expanded);
    }
}
