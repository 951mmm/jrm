use quote::{format_ident, quote};
use syn::{
    Attribute, Expr, Field, Fields, FieldsNamed, FieldsUnnamed, GenericArgument, Ident, Item,
    ItemEnum, ItemStruct, PathArguments, Type, TypePath, bracketed, parenthesized,
};

use base_macro::{attr_enum, simple_field_attr, syn_err};

pub fn class_file_parse_derive_inner(ast: &Item) -> syn::Result<proc_macro2::TokenStream> {
    let result = match &ast {
        Item::Struct(item_struct) => resolve_struct(item_struct)?,
        Item::Enum(item_enum) => resolve_enum(item_enum)?,
        _ => unreachable!(),
    };

    Ok(result)
}

fn resolve_struct(item_struct: &ItemStruct) -> syn::Result<proc_macro2::TokenStream> {
    let ItemStruct { fields, ident, .. } = item_struct;
    let result = match fields {
        Fields::Named(fields_named) => resolve_named(fields_named, ident)?,
        Fields::Unnamed(fields_unnamed) => resolve_unnamed(fields_unnamed, ident)?,
        _ => unreachable!(),
    };
    Ok(result)
}

fn resolve_enum(item_enum: &ItemEnum) -> syn::Result<proc_macro2::TokenStream> {
    let ItemEnum {
        variants,
        ident,
        attrs,
        ..
    } = item_enum;
    let attr_enum_entry = attr_enum_entry(attrs)?;
    if let EnumEntry::Index {
        index_ty,
        map_ident,
    } = &attr_enum_entry
    {
        let mut arm_expr = vec![];
        for variant in variants {
            let variant_ident = &variant.ident;
            let fields = &variant.fields;
            let lit = variant.ident.to_string();
            let expr = get_match_arms(fields, ident, variant_ident, &lit)?;
            arm_expr.push(expr);
        }

        let debug_token_stream = if cfg!(feature = "debug") {
            quote! {println!("enum is: {}, index is: {}",stringify!(#ident), index);}
        } else {
            quote! {}
        };
        return Ok(quote! {
            impl ClassParser for #ident {
                fn parse(ctx: &mut ParserContext) -> anyhow::Result<Self> {
                    let index = <#index_ty as ClassParser>::parse(ctx)?;
                    #debug_token_stream
                    ctx.enum_entry = Box::new(index);
                    let choice: String = ContextIndex::get(&ctx.#map_ident, index);
                    let result = match choice.as_str() {
                        #(#arm_expr,)*
                        _ => {
                            unreachable!()
                        }
                    };
                    return Ok(result);
                }
            }
        });
    }
    syn_err!(item_enum, "failed to parse enum!");
}

fn get_match_arms(
    fields: &Fields,
    enum_ident: &Ident,
    variant_ident: &Ident,
    lit: &str,
) -> syn::Result<proc_macro2::TokenStream> {
    let constructor = quote! {#enum_ident::#variant_ident};
    let mut temp_idents = vec![];
    let mut parse_stmts = vec![];

    match fields {
        Fields::Unnamed(fields_unnamed) => {
            for (index, field) in (&fields_unnamed.unnamed).into_iter().enumerate() {
                let field_ty = &field.ty;
                let temp_ident = format_ident!("temp_{}", index);

                let debug_token_stream = if cfg!(feature = "debug") {
                    quote! {println!("val is: {:?}", #temp_ident);}
                } else {
                    quote! {}
                };
                let stmt = quote! {
                    let #temp_ident = <#field_ty as ClassParser>::parse(ctx)?;
                    #debug_token_stream
                };
                temp_idents.push(temp_ident);
                parse_stmts.push(stmt);
            }
            Ok(quote! {
                #lit => {
                    #(#parse_stmts)*
                    #constructor(#(#temp_idents),*)
                }
            })
        }
        Fields::Unit => Ok(quote! {
            #lit => #constructor
        }),
        _ => {
            syn_err!(
                enum_ident,
                "invalid field type, surpport `unamed` or `unit`"
            );
        }
    }
}
fn resolve_named(
    fields_named: &FieldsNamed,
    struct_ident: &Ident,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut field_idents = vec![];
    let mut parse_stmts = vec![];
    let mut collection_impl_blocks = vec![];

    for field in &fields_named.named {
        let field_ident = &field.ident;
        let field_ty = &field.ty;

        let count = attr_count(field)?;
        let constant_index = attr_constant_index(field)?;
        let enum_entry = attr_enum_entry(&field.attrs)?;
        // NOTE set和read或许不会同时出现。需要包装
        let is_set_constant_pool = attr_constant_pool(field)?.eq(&ConstantPool::Set);
        let is_constant_pool_read_mode = attr_constant_pool(field)?.eq(&ConstantPool::Read);
        let skip_expr = attr_skip(field)?;

        match skip_expr {
            None => {
                let mut stmt = quote! {
                    let #field_ident = <#field_ty as ClassParser>::parse(ctx)?;
                };
                if let EnumEntry::Get = enum_entry {
                    stmt = quote! {
                        let #field_ident = ctx.enum_entry.downcast_ref::<#field_ty>().unwrap().clone();
                    };
                }
                parse_stmts.push(stmt);

                match count {
                    Count::Get => {
                        if is_constant_pool_read_mode {
                            collection_impl_blocks.push(resolve_collection_impl(field_ty, true)?);
                        } else {
                            collection_impl_blocks.push(resolve_collection_impl(field_ty, false)?);
                        }
                    }
                    Count::Set => {
                        parse_stmts.push(quote! {
                            ctx.count = #field_ident as usize;
                        });
                    }
                    _ => {}
                };
                match constant_index {
                    ConstantIndex::Setend => {
                        parse_stmts.push(quote! {
                            ctx.constant_index_range = 1..#field_ident;
                        });
                    }
                    ConstantIndex::Check => {
                        parse_stmts.push(quote! {
                    if !ctx.constant_index_range.contains(&#field_ident) {
                        anyhow::bail!("invalid {}, not in range {:?}", stringify!(#field_ident), ctx.constant_index_range);
                    }
                });
                    }
                    _ => {}
                }

                if is_set_constant_pool {
                    parse_stmts.push(quote! {
                        ctx.constant_pool = #field_ident.clone();
                    });
                }
            }
            Some(expr) => {
                parse_stmts.push(quote! {
                    let #field_ident = #expr;
                });
            }
        }
        field_idents.push(field_ident);
    }

    Ok(quote! {
        impl ClassParser for #struct_ident {
            fn parse(ctx: &mut ParserContext) -> anyhow::Result<Self> {
                #(#parse_stmts)*
                Ok(Self {
                    #(#field_idents,)*
                })
            }
        }
        #(#collection_impl_blocks)*
    })
}

fn resolve_unnamed(
    fields_unnamed: &FieldsUnnamed,
    struct_ident: &Ident,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut temp_idents = vec![];
    let mut parse_stmts = vec![];
    let mut collection_impl_block = None;

    for (index, field) in (&fields_unnamed.unnamed).into_iter().enumerate() {
        let field_ty = &field.ty;
        let temp_ident = format_ident!("temp_{}", index);

        let is_get_count = attr_count(field)?.eq(&Count::Get);
        let is_constant_pool_read_mode = attr_constant_pool(field)?.eq(&ConstantPool::Read);
        let stmt = quote! {
            let #temp_ident = <#field_ty as ClassParser>::parse(ctx)?;
        };

        if is_get_count {
            collection_impl_block = Some(resolve_collection_impl(
                field_ty,
                is_constant_pool_read_mode,
            )?);
        }
        parse_stmts.push(stmt);
        temp_idents.push(temp_ident);
    }
    let result = quote! {
        impl ClassParser for #struct_ident {
            fn parse(ctx: &mut ParserContext) -> anyhow::Result<Self> {
                #(#parse_stmts)*
                Ok(#struct_ident(#(#temp_idents),*))
            }
        }
        #collection_impl_block
    };
    Ok(result)
}

#[attr_enum]
enum ConstantPool {
    Set,
    Read,
}

#[attr_enum]
enum ConstantIndex {
    Setend,
    Check,
}

#[attr_enum]
enum Count {
    Set,
    Get,
    Impled,
}
// #[attr_enum]
enum EnumEntry {
    Get,
    Index { index_ty: Type, map_ident: Ident },
    None,
}

fn attr_enum_entry(attrs: &Vec<Attribute>) -> syn::Result<EnumEntry> {
    let mut enum_entry = EnumEntry::None;
    for attr in attrs {
        if attr.path().is_ident("enum_entry") {
            attr.parse_nested_meta(|meta| {
                // #[enum_entry(get)]
                if meta.path.is_ident("get") {
                    enum_entry = EnumEntry::Get;
                    return Ok(());
                }
                // #[enum_entry(index(map[ty]))]
                if meta.path.is_ident("index") {
                    let content;
                    parenthesized!(content in meta.input);
                    let ident: Ident = content.parse()?;
                    let content_in_square;
                    bracketed!(content_in_square in content);
                    let ty: Type = content_in_square.parse()?;
                    enum_entry = EnumEntry::Index {
                        index_ty: ty,
                        map_ident: ident,
                    };
                    return Ok(());
                }
                Err(meta.error("unrecongnized enum_entry"))
            })?;
        }
    }
    Ok(enum_entry)
}

fn attr_skip(field: &Field) -> syn::Result<Option<Expr>> {
    field
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("skip"))
        .map(|attr| attr.parse_args())
        .transpose()
}

fn resolve_collection_impl(
    collection_ty: &Type,
    is_constant_pool: bool,
) -> syn::Result<proc_macro2::TokenStream> {
    let collection_ident = get_collection_ident(collection_ty)?;
    let inner_ty = get_inner_ty(collection_ty)?;
    let stmts = if is_constant_pool {
        quote! {
            let mut collection = #collection_ident::with_capacity(size);
            let invalid = Constant::Invalid;
            collection.push(invalid);
            for _ in 0..size-1 {
                let item = <#inner_ty as ClassParser>::parse(ctx)?;
                collection.push(item);
            }
            return Ok(collection);

        }
    } else {
        quote! {
            let mut collection = #collection_ident::with_capacity(size);
            for _ in 0..size {
                let item = <#inner_ty as ClassParser>::parse(ctx)?;
                collection.push(item);
            }
            return Ok(collection);
        }
    };
    Ok(quote! {
        impl ClassParser for #collection_ty {
            fn parse(ctx: &mut ParserContext) -> anyhow::Result<Self> {
                let size = ctx.count.clone();
                #stmts
            }
        }
    })
}
fn get_collection_ident(ty: &Type) -> syn::Result<&Ident> {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            return Ok(&segment.ident);
        }
    }
    syn_err! {ty, "failed to get collection ty"};
}
fn get_inner_ty(ty: &Type) -> syn::Result<&Type> {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            if let PathArguments::AngleBracketed(arg) = &segment.arguments {
                for arg in &arg.args {
                    if let GenericArgument::Type(inner_ty) = arg {
                        return Ok(inner_ty);
                    }
                }
            }
        }
    }
    syn_err! {ty, "Invalid inner type for Vec"};
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use macro_utils::print_expanded_fmt;
    use quote::ToTokens;
    use syn::{Ident, Item, ItemEnum, Type, Variant, parse_quote};

    use crate::{
        class_file_parse_derive_inner,
        class_parser::{EnumEntry, attr_enum_entry, get_match_arms, resolve_enum},
    };

    #[test]
    fn test_resolve_struct_named_set_constant_pool_attr_set_constant_pool()
    -> Result<(), Box<dyn Error>> {
        let code: Item = parse_quote!(
            #[derive(ClassParser)]
            struct TestStruct {
                a: u8,
                #[constant_pool(set)]
                b: ConstantPool,
            }
        );
        let expanded = class_file_parse_derive_inner(&code)?;
        let raw_code = expanded.to_string();
        assert!(raw_code.contains("ctx . constant_pool ="));
        print_expanded_fmt(expanded);
        Ok(())
    }

    #[test]
    fn test_attr_enum_entry() -> Result<(), Box<dyn Error>> {
        let attrs = vec![
            parse_quote!(#[enum_entry(get)]),
            parse_quote!(#[enum_entry(index(map[u8]))]),
        ];

        let mut results = attrs.iter().cloned().map(|attr| {
            let attrs = vec![attr];
            attr_enum_entry(&attrs).unwrap()
        });

        assert!(matches!(results.next().unwrap(), EnumEntry::Get));
        assert!(test_attr_enum_entry_1(&results.next().unwrap()));

        Ok(())
    }
    fn test_attr_enum_entry_1(result: &EnumEntry) -> bool {
        let u8_ty: Type = parse_quote!(u8);
        if let EnumEntry::Index {
            index_ty,
            map_ident,
        } = result
        {
            if ty_eq(index_ty, &u8_ty) && map_ident == "map" {
                return true;
            }
        }
        false
    }

    fn ty_eq(tyl: &Type, tyr: &Type) -> bool {
        tyl.to_token_stream()
            .to_string()
            .eq(&tyr.to_token_stream().to_string())
    }

    #[test]
    fn test_get_match_arms_unamed_expand() -> Result<(), Box<dyn Error>> {
        let variant: Variant = parse_quote!(V(a));
        let raw_code = generate_match_arms(&variant)?;
        assert!(raw_code.contains("\"lit\" =>"));
        assert!(raw_code.contains("TestEnum :: V (temp_0)"));
        println!("{}", raw_code);
        Ok(())
    }
    #[test]
    fn test_get_match_arms_unit_expand() -> Result<(), Box<dyn Error>> {
        let variant: Variant = parse_quote!(Somee);
        let raw_code = generate_match_arms(&variant)?;
        assert!(raw_code.contains("TestEnum :: Somee"));
        println!("{}", raw_code);
        Ok(())
    }
    #[test]
    fn test_get_match_arms_named_expand() {
        let varaint: Variant = parse_quote!(A { b: B });
        let err = generate_match_arms(&varaint).unwrap_err();
        assert_eq!(
            err.to_string(),
            "invalid field type, surpport `unamed` or `unit`"
        );
    }
    fn generate_match_arms(variant: &Variant) -> syn::Result<String> {
        let ident: Ident = parse_quote!(TestEnum);
        let expanded = get_match_arms(&variant.fields, &ident, &variant.ident, "lit")?;
        let raw_code = expanded.to_string();
        Ok(raw_code)
    }
    #[test]
    fn test_resolve_enum_expand() -> Result<(), Box<dyn Error>> {
        let code: ItemEnum = parse_quote! {
            #[enum_entry(index(map[u8]))]
            enum TestEnum {
                A(a),
                B
            }
        };
        let expanded = resolve_enum(&code)?;
        let raw_code = expanded.to_string();
        assert!(raw_code.contains("let index = "));
        assert!(raw_code.contains("ContextIndex :: get (& ctx . map , index) ? ;"));
        print_expanded_fmt(expanded);
        Ok(())
    }
}
