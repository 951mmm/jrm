mod base_attribbute;
mod class_parser;
mod klass_debug;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, Parser};
use syn::punctuated::Punctuated;
use syn::{
    DeriveInput, Ident, Item, ItemStruct, Stmt, Token, Type, parse_macro_input, parse_quote,
};

use base_macro::unwrap_err;

use crate::base_attribbute::base_attrubute_inner;
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

#[proc_macro_derive(ClassParser, attributes(count, constant_pool, constant_index, ahead))]
pub fn class_file_parse_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as Item);
    unwrap_err!(class_file_parse_derive_inner(&ast))
}

#[proc_macro_attribute]
pub fn base_attribute(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as base_attribbute::Attrs);
    let mut item_struct = parse_macro_input!(item as ItemStruct);
    unwrap_err!(base_attrubute_inner(&attrs, &mut item_struct))
}

// struct DefineAttributes {
//     stmts: Vec<Stmt>,
// }

// impl Parse for DefineAttributes {
//     fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
//         let mut stmts = vec![];
//         while !input.is_empty() {
//             stmts.push(input.parse::<Stmt>()?);
//         }
//         Ok(Self { stmts })
//     }
// }
// #[proc_macro]
// pub fn define_attributes(input: TokenStream) -> TokenStream {
//     let mut stmts = parse_macro_input!(input as DefineAttributes).stmts;
//     for stmt in stmts.iter_mut() {
//         if let Stmt::Item(item) = stmt {
//             if let Item::Struct(item_struct) = item {
//                 item_struct.attrs.push(parse_quote!(#[base_attribute]));
//             }
//         }
//     }

//     quote! {
//         #(#stmts)*
//     }
//     .into()
// }
struct AttributeEnum {
    idents: Punctuated<Ident, Token![,]>,
}
impl Parse for AttributeEnum {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let idents = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;
        Ok(AttributeEnum { idents })
    }
}
#[proc_macro]
pub fn attribute_enum(input: TokenStream) -> TokenStream {
    let attribute_enum = parse_macro_input!(input as AttributeEnum);

    let variants = attribute_enum.idents.iter().map(|ident| {
        let attribute_ident = format_ident!("{}Attribute", ident);
        quote! {
            #ident(#attribute_ident)
        }
    });

    quote! {
        pub enum Attribute {
            #(#variants),*
        }
    }
    .into()
}
