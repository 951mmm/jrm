mod class_parser;
mod klass_debug;

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident, Item, Type, parse_macro_input, parse_quote};

use crate::class_parser::class_file_parse_derive_inner;
use crate::klass_debug::klass_debug_derive_inner;
#[proc_macro]
pub fn generate_ux(_: TokenStream) -> TokenStream {
    let parse_expr: Vec<(Ident, Type)> = vec![
        (parse_quote!(read_one_byte), parse_quote!(u8)),
        (parse_quote!(read_two_bytes), parse_quote!(u16)),
        (parse_quote!(read_four_bytes), parse_quote!(u32)),
    ];
    let parse_stmts = parse_expr.iter().map(|(call, ty)| {
        quote! {
            impl ClassParser for #ty {
                fn parse(class_reader: &mut ClassReader, _: &mut ParserContext) -> anyhow::Result<Self> {
                    Ok(class_reader.#call().unwrap_or(0))
                }
            }
        }
    });
    let from_stmts = parse_expr.iter().map(|(_, ty)| {
        quote! {
            impl std::convert::From<#ty> for StoreType {
                fn from(value: #ty) -> StoreType {
                    StoreType::Usize(value as usize)
                }
            }
        }
    });

    quote! {
        #(#parse_stmts)*
        #(#from_stmts)*
    }
    .into()
}

macro_rules! unwrap_err {
    ($call_expr: expr) => {
        match $call_expr {
            Ok(tokens) => tokens.into(),
            Err(err) => err.to_compile_error().into(),
        }
    };
}

#[proc_macro_derive(KlassDebug, attributes(hex))]
pub fn klass_debug_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    unwrap_err!(klass_debug_derive_inner(&ast))
}

#[proc_macro_derive(ClassParser, attributes(impl_sized, set_ctx, get_ctx, constant_pool))]
pub fn class_file_parse_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as Item);
    unwrap_err!(class_file_parse_derive_inner(&ast))
}
