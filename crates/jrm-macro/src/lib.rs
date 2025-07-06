mod base_attribute;
mod build_enum_input;
mod class_parser;
mod constant;
mod constant_enum;
mod define_constants;
mod impl_class_parser_for_vec;
mod klass_debug;
mod utils;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Ident, Item, ItemStruct, Type, parse_macro_input, parse_quote};

use base_macro::unwrap_err;

use crate::base_attribute::base_attrubute_inner;
use crate::class_parser::class_file_parse_derive_inner;
use crate::constant::constant_inner;
use crate::constant_enum::constant_enum_inner;
use crate::define_constants::define_constants_inner;
use crate::impl_class_parser_for_vec::impl_class_parser_for_vec_inner;
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

    quote! {
        #(#parse_stmts)*
    }
    .into()
}

#[proc_macro_derive(KlassDebug, attributes(hex))]
pub fn klass_debug_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    unwrap_err!(klass_debug_derive_inner(&ast))
}

#[proc_macro_derive(
    ClassParser,
    attributes(count, constant_pool, constant_index, enum_entry)
)]
pub fn class_file_parse_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as Item);
    unwrap_err!(class_file_parse_derive_inner(&ast))
}

#[proc_macro_attribute]
pub fn base_attribute(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as base_attribute::Attrs);
    let mut item_struct = parse_macro_input!(item as ItemStruct);
    unwrap_err!(base_attrubute_inner(&attrs, &mut item_struct))
}

#[proc_macro]
pub fn attribute_enum(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as build_enum_input::Ast);

    let variants = ast.idents.iter().map(|ident| {
        let attribute_ident = format_ident!("{}Attribute", ident);
        quote! {
            #ident(#attribute_ident)
        }
    });

    quote! {
        #[derive(Debug, ClassParser)]
        #[enum_entry(index(constant_pool[u16]))]
        pub enum Attribute {
            #(#variants),*
        }
    }
    .into()
}

#[proc_macro]
pub fn impl_class_parser_for_vec(input: TokenStream) -> TokenStream {
    let ty = parse_macro_input!(input as Type);
    unwrap_err!(impl_class_parser_for_vec_inner(&ty))
}

#[proc_macro]
pub fn define_constants(input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as define_constants::Ast);
    define_constants_inner(&mut ast.structs).into()
}
#[proc_macro]
pub fn constant_enum(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as build_enum_input::Ast);
    constant_enum_inner(&ast).into()
}
#[proc_macro_attribute]
pub fn constant(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as constant::Attrs);
    let mut item_struct = parse_macro_input!(input as ItemStruct);
    constant_inner(&attr, &mut item_struct).into()
}
