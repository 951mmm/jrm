use std::collections::HashMap;

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Expr, Fields, FieldsNamed, GenericArgument, Ident,
    LitInt, LitStr, PathArguments, Token, Type, TypePath, parse::Parse, parse_macro_input,
    parse_quote,
};

struct ParseIndex {
    class_reader_ident: Ident,
    variable_ident: Ident,
    bytes_cnt: LitInt,
    error_formater: Expr,
}

impl Parse for ParseIndex {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let class_reader_ident: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let variable_ident: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let bytes_cnt: LitInt = input.parse()?;
        input.parse::<Token![,]>()?;
        let error_formater: Expr = input.parse()?;
        Ok(ParseIndex {
            class_reader_ident,
            variable_ident,
            bytes_cnt,
            error_formater,
        })
    }
}
#[proc_macro]
pub fn parse_not_zero(input: TokenStream) -> TokenStream {
    // Placeholder implementation
    let ast = parse_macro_input!(input as ParseIndex);
    let class_reader_ident = &ast.class_reader_ident;
    let variable_ident = &ast.variable_ident;
    let bytes_cnt = ast.bytes_cnt.base10_parse::<u16>().unwrap();
    let error_formater = &ast.error_formater;

    let read_bytes_expr = match bytes_cnt {
        1 => {
            quote! {
                #class_reader_ident.read_one_byte().unwrap_or(0);
            }
        }
        2 => {
            quote! {
                #class_reader_ident.read_two_bytes().unwrap_or(0);
            }
        }
        4 => {
            quote! {
                #class_reader_ident.read_four_bytes().unwrap_or(0);
            }
        }
        _ => {
            quote! {
                #class_reader_ident.read_bytes(#bytes_cnt).unwrap_or(vec![0; #bytes_cnt as usize]);
            }
        }
    };
    quote! {
        let #variable_ident = #read_bytes_expr;
        if #variable_ident == 0 {
            anyhow::bail!(#error_formater);
        }
    }
    .into()
}

#[proc_macro_derive(KlassDebug, attributes(hex))]
pub fn klass_debug_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    match klass_debug_derive_inner(&ast) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn klass_debug_derive_inner(ast: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let klass_ident = &ast.ident;
    let mut field_stmts = vec![];

    if let Data::Struct(DataStruct { fields, .. }) = &ast.data {
        for field in fields {
            let field_ident = &field.ident;
            let field_ident_lit = field_ident.clone().unwrap().to_string();
            let title_case_lit = field_ident_lit
                .as_str()
                .from_case(Case::Snake)
                .to_case(Case::Title);

            let is_hex = field.attrs.iter().any(|attr| attr.path().is_ident("hex"));

            // let is_vec = if let Type::Path(TypePath { path, .. }) = field_ty {
            //     let last_segment = path.segments.last();
            //     match last_segment {
            //         Some(segment) if segment.ident == "Vec" => true,
            //         _ => false,
            //     }
            // } else {
            //     false
            // };

            if is_hex {
                field_stmts.push(quote! {
                    writeln!(f, "  {}: 0X{:X}", #title_case_lit, self.#field_ident)?;
                });
            } else {
                field_stmts.push(quote! {
                    writeln!(f, "  {}: {:?}", #title_case_lit, self.#field_ident)?;
                });
            }
        }
    }

    let result = quote! {
        impl std::fmt::Debug for #klass_ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "InstantKlass:\n")?;
                #(#field_stmts;)*
                Ok(())
            }
        }
    };

    Ok(result)
}

#[proc_macro_derive(ClassFileParse, attributes(turple_fn, index, property))]
pub fn class_file_parse_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    match class_file_parse_derive_inner(&ast) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
fn class_file_parse_derive_inner(ast: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_ident = &ast.ident;
    let mut field_idents = vec![];
    let mut parse_stmts = vec![];
    let mut turple_fn_map = HashMap::new();
    let mut turple_fn_index_map = HashMap::new();
    let mut parse_fns = vec![];

    if let Data::Struct(DataStruct { fields, .. }) = &ast.data {
        for (index, field) in fields.iter().enumerate() {
            let field_ident = &field.ident;
            let field_ty = &field.ty;

            let mut is_turple_fn = false;
            let mut is_index = false;
            let mut is_property = false;

            for attr in &field.attrs {
                if attr.path().is_ident("turple_fn") {
                    is_turple_fn = true;
                    if let Ok(fn_ident) = attr.parse_args::<Ident>() {
                        let key = fn_ident.to_string();
                        if !turple_fn_index_map.contains_key(&key) {
                            turple_fn_index_map.insert(key.clone(), index);
                        }
                        if turple_fn_map.contains_key(&key) {
                            let turple_list: &mut Vec<&Option<Ident>> =
                                turple_fn_map.get_mut(&key).unwrap();
                            turple_list.push(field_ident);
                        } else {
                            let turple_list = vec![field_ident];
                            turple_fn_map.insert(key, turple_list);
                        }
                    }
                } else if attr.path().is_ident("index") {
                    is_index = true;
                } else if attr.path().is_ident("property") {
                    is_property = true;
                }
            }

            let parse_fn_ident = format_ident!("parse_{}", field_ident.as_ref().unwrap());
            if !is_turple_fn {
                if is_index {
                    parse_fns.push(quote! {
                        fn #parse_fn_ident(class_reader: &mut ClassReader) -> anyhow::Result<#field_ty> {
                            parse_not_zero!(class_reader, #field_ident, 2, format!("Invalid {}", stringify!(#field_ident)));
                            Ok(#field_ident)
                        }
                    });
                }
                parse_stmts.push(quote! {
                    let #field_ident = Self::#parse_fn_ident(class_reader)?;
                });
            } else {
                parse_stmts.push(quote! {});
            }
            if is_property {
                let inner_ty = get_vec_inner_ty(field_ty)?;
                let item_ident_lit = inner_ty
                    .to_token_stream()
                    .to_string()
                    .from_case(Case::Camel)
                    .to_case(Case::Snake);
                let item_ident = format_ident!("{}", item_ident_lit);
                parse_fns.push(quote! {
                        fn #parse_fn_ident(class_reader: &mut ClassReader) -> anyhow::Result<(u16, #field_ty)> {
                            let count = class_reader.read_two_bytes().unwrap_or(0);
                            let mut #field_ident = std::vec::Vec::with_capacity(count as usize);
                            for _ in 0..count {
                                let #item_ident = #inner_ty(Self::parse_property(class_reader)?);
                                fields.push(#item_ident);
                            }
                            Ok((count, #field_ident))
                        }
                    });
            }

            field_idents.push(field_ident.clone());
        }
    }
    for (turple_fn_lit, turple_list) in &turple_fn_map {
        let turple_fn_ident = format_ident!("{}", turple_fn_lit);
        let index = turple_fn_index_map.get(turple_fn_lit).unwrap().clone();

        parse_stmts[index] = quote! {
            let (#(#turple_list),*) = Self::#turple_fn_ident(class_reader)?;
        };
    }
    Ok(quote! {
        impl ClassFileParser {
            pub fn parse(class_reader: &mut ClassReader) -> anyhow::Result<#struct_ident> {
                #(#parse_stmts)*
                Ok(#struct_ident {
                    #(#field_idents,)*
                })
            }
            #(#parse_fns)*
        }
    })
}

fn get_vec_inner_ty(ty: &Type) -> syn::Result<&Type> {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            if segment.ident == "Vec" {
                if let PathArguments::AngleBracketed(arg) = &segment.arguments {
                    for arg in &arg.args {
                        if let GenericArgument::Type(inner_ty) = arg {
                            return Ok(inner_ty);
                        }
                    }
                }
            }
        }
    }
    Err(syn::Error::new_spanned(ty, "Invalid inner type for Vec"))
}

#[proc_macro_derive(ConstantConstuctor, attributes(ignored))]
pub fn constant_constructor_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    match constant_constructor_derive_inner(&ast) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
fn constant_constructor_derive_inner(ast: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_ident = &ast.ident;
    let mut constructor_fns = vec![];

    if let Data::Enum(DataEnum { variants, .. }) = &ast.data {
        for variant in variants {
            let variant_ident = &variant.ident;
            let constructor_fn_ident = format_ident!(
                "new_{}",
                variant_ident
                    .to_string()
                    .from_case(Case::Camel)
                    .to_case(Case::Snake)
            );
            let mut temp_idents = vec![];
            let mut temp_tys = vec![];

            let is_ignored = variant
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("ignored"));

            if is_ignored {
                continue;
            }

            match &variant.fields {
                Fields::Named(_) => {
                    unreachable!()
                }
                Fields::Unnamed(fields_unnamed) => {
                    let mut index = 0_usize;
                    for field in &fields_unnamed.unnamed {
                        let field_ty = &field.ty;
                        let temp_ident = format_ident!("temp_{}", index);
                        temp_idents.push(temp_ident.clone());
                        temp_tys.push(field_ty.clone());
                        index += 1;
                    }

                    let constructor_fn_args =
                        temp_idents.iter().zip(temp_tys.iter()).map(|(ident, ty)| {
                            quote! {
                                #ident: #ty
                            }
                        });

                    constructor_fns.push(quote! {
                        pub fn #constructor_fn_ident(tag: u8, #(#constructor_fn_args),*) -> ConstantWrapper {
                            ConstantWrapper {
                                tag,
                                constant: #struct_ident::#variant_ident(#(#temp_idents),*)}
                        }
                    });
                }
                Fields::Unit => {
                    constructor_fns.push(quote! {
                        pub fn #constructor_fn_ident(tag: u8) -> ConstantWrapper {
                            ConstantWrapper {
                                tag,
                                constant: #struct_ident::#variant_ident
                            }
                        }
                    });
                }
            }
        }
    }
    Ok(quote! {
        impl ConstantWrapper {
            #(#constructor_fns)*
        }
    })
}
