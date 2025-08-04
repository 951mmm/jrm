mod attr_enum;
mod syn_err;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    DeriveInput, Expr, LitStr, parse_macro_input,
};

use syn_err::syn_err_inner;

use crate::attr_enum::attr_enum_inner;

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
#[proc_macro]
pub fn unwrap_darling(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);
    quote! {
        match #expr {
            Ok(tokens) => tokens,
            Err(err) => return err.write_errors().into(),
        }
    }
    .into()
}
#[proc_macro]
pub fn syn_err(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn_err::Args);
    syn_err_inner(&ast).into()
}

#[proc_macro_attribute]
pub fn attr_enum(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as attr_enum::Attrs);
    let mut ast = parse_macro_input!(input as DeriveInput);
    attr_enum_inner(&attrs, &mut ast).into()
}
