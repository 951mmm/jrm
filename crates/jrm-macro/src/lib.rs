mod class_parser;
mod klass_debug;

use proc_macro::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{
    DeriveInput, Field, Fields, Ident, Item, ItemStruct, Token, Type, parse_macro_input,
    parse_quote,
};

use base_macro::unwrap_err;

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
    attributes(impl_sized, count, constant_pool, constant_index, ahead)
)]
pub fn class_file_parse_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as Item);
    unwrap_err!(class_file_parse_derive_inner(&ast))
}

#[proc_macro_attribute]
pub fn base_attribute(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let base_fields_prefix = [
        quote! {attribute_name_index: u16},
        quote! {attribute_length: u32},
    ];
    let base_fields_suffix = [
        quote! {attributes_count: u16},
        quote! {attributes: Vec<Attribute>},
    ];
    let mut item_struct = parse_macro_input!(item as ItemStruct);

    if let Fields::Named(ref mut field_named) = item_struct.fields {
        let mut new_named: Punctuated<Field, Token![,]> = Punctuated::new();
        for base_field in base_fields_prefix {
            new_named.push(parse_quote!(#base_field));
        }
        new_named.extend(field_named.named.clone());
        for base_field in base_fields_suffix {
            new_named.push(parse_quote!(#base_field));
        }
        field_named.named = new_named;
    }
    quote! {
        #item_struct
    }
    .into()
}
