mod enum_entry;

use quote::{format_ident, quote};
use syn::{
    Fields, FieldsNamed, FieldsUnnamed, GenericArgument, Ident, Item, ItemEnum, ItemStruct,
    PathArguments, Type, TypePath,
};

use base_macro::{attr_enum, simple_field_attr};
use macro_utils::{FromAttrs, syn_err};

use enum_entry::EnumEntry;

use crate::class_parser::enum_entry::IndexMeta;

pub fn derive_class_parser_inner(ast: &Item) -> syn::Result<proc_macro2::TokenStream> {
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
        Fields::Named(fields_named) => resolve_struct_fields_named(fields_named, ident)?,
        Fields::Unnamed(fields_unnamed) => resolve_struct_fields_unnamed(fields_unnamed, ident)?,
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
    let attr_enum_entry = EnumEntry::from_attrs(attrs)?;
    if let EnumEntry::Index(index_mata) = &attr_enum_entry {
        let IndexMeta {
            index_ty,
            map_ident,
            outer,
        } = index_mata.as_ref();
        let mut arm_expr = vec![];
        for variant in variants {
            let variant_ident = &variant.ident;
            let fields = &variant.fields;
            let lit = variant.ident.to_string();
            let expr = get_match_arms(fields, ident, variant_ident, &lit)?;
            arm_expr.push(expr);
        }

        let index_stmts = if *outer {
            generate_entry_get_stmt(index_ty, &Some(format_ident!("index")))
        } else {
            quote! {
                let index = <#index_ty as ClassParser>::parse(ctx)?;
                ctx.enum_entry = Box::new(index);
            }
        };

        let debug_token_stream = if cfg!(feature = "debug") {
            quote! {println!("enum is: {}, index is: {}",stringify!(#ident), index);}
        } else {
            quote! {}
        };
        return Ok(quote! {
            impl ClassParser for #ident {
                fn parse(ctx: &mut ParserContext) -> anyhow::Result<Self> {
                    #index_stmts
                    #debug_token_stream
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

    match fields {
        Fields::Named(fields_named) => resolve_enum_fields_named(fields_named, lit, &constructor),
        Fields::Unnamed(fields_unnamed) => {
            resolve_enum_fields_unnamed(fields_unnamed, lit, &constructor)
        }

        Fields::Unit => Ok(quote! {
            #lit => #constructor
        }),
    }
}

fn resolve_enum_fields_named(
    fields_named: &FieldsNamed,
    lit: &str,
    constructor: &proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut parse_stmts = vec![];
    let mut field_idents = vec![];
    for field in &fields_named.named {
        let field_ident = &field.ident;
        let field_ty = &field.ty;
        parse_stmts.push(quote! {
            let #field_ident = <#field_ty as ClassParser>::parse(ctx)?;

        });
        field_idents.push(field_ident);
    }
    Ok(quote! {
        #lit => {
            #(#parse_stmts)*
            #constructor {
                #(#field_idents),*
            }
        }
    })
}

fn resolve_enum_fields_unnamed(
    fields_unnamed: &FieldsUnnamed,
    lit: &str,
    constructor: &proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut temp_idents = vec![];
    let mut parse_stmts = vec![];
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

fn resolve_struct_fields_named(
    fields_named: &FieldsNamed,
    struct_ident: &Ident,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut field_idents = vec![];
    let mut parse_stmts = vec![];
    let mut collection_impl_blocks = vec![];

    for field in &fields_named.named {
        let field_ident = &field.ident;
        let field_ty = &field.ty;

        let field_attrs = &field.attrs;
        let count = Count::from_attrs(field_attrs)?;
        let constant_index = ConstantIndex::from_attrs(field_attrs)?;
        let enum_entry = EnumEntry::from_attrs(field_attrs)?;
        // NOTE set和read或许不会同时出现。需要包装
        let is_set_constant_pool =
            matches!(ConstantPool::from_attrs(field_attrs)?, ConstantPool::Set);
        let is_constant_pool_read_mode =
            matches!(ConstantPool::from_attrs(field_attrs)?, ConstantPool::Read);
        let is_skip = skip_from_attrs(field_attrs);

        match is_skip {
            false => {
                let stmt = quote! {
                    let #field_ident = <#field_ty as ClassParser>::parse(ctx)?;
                };
                let stmt = match enum_entry {
                    EnumEntry::Get => generate_entry_get_stmt(field_ty, field_ident),
                    EnumEntry::Set => {
                        quote! {
                            #stmt
                            ctx.enum_entry = Box::new(#field_ident);
                        }
                    }
                    EnumEntry::Index(_) => {
                        syn_err!(
                            field,
                            "invalid enum entry `index`, expected `get` and `set` for struct field"
                        );
                    }
                    EnumEntry::None => stmt,
                };

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
            true => {
                parse_stmts.push(quote! {
                    let #field_ident = std::default::Default::default();
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

fn resolve_struct_fields_unnamed(
    fields_unnamed: &FieldsUnnamed,
    struct_ident: &Ident,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut temp_idents = vec![];
    let mut parse_stmts = vec![];
    let mut collection_impl_block = None;

    for (index, field) in (&fields_unnamed.unnamed).into_iter().enumerate() {
        let field_ty = &field.ty;
        let temp_ident = format_ident!("temp_{}", index);
        let field_attrs = &field.attrs;

        let is_get_count = matches!(Count::from_attrs(field_attrs)?, Count::Get);
        let is_constant_pool_read_mode =
            matches!(ConstantPool::from_attrs(field_attrs)?, ConstantPool::Read);
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

#[attr_enum(class_parser)]
enum ConstantPool {
    Set,
    Read,
}

#[attr_enum(class_parser)]
enum ConstantIndex {
    Setend,
    Check,
}

#[attr_enum(class_parser)]
enum Count {
    Set,
    Get,
    Impled,
}
// #[attr_enum]

simple_field_attr! {class_parser(skip)}

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
    syn_err!(ty, "failed to get collection ty");
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
    syn_err!(ty, "invalid inner type for Vec");
}

fn generate_entry_get_stmt(ty: &Type, index_ident: &Option<Ident>) -> proc_macro2::TokenStream {
    quote! {
        let #index_ident = ctx.enum_entry
            .downcast_ref::<#ty>()
            .ok_or(anyhow::anyhow!("failed to get entry index with type: {}", stringify!(#ty)))?
            .clone();
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use macro_utils::print_expanded_fmt;

    use syn::{Ident, Item, ItemEnum, Variant, parse_quote};

    use crate::{
        class_parser::{get_match_arms, resolve_enum},
        derive_class_parser_inner,
    };

    #[test]
    fn test_resolve_struct_named_set_constant_pool_attr_set_constant_pool()
    -> Result<(), Box<dyn Error>> {
        let code: Item = parse_quote!(
            #[derive(ClassParser)]
            struct TestStruct {
                a: u8,
                #[class_parser(constant_pool(set))]
                b: ConstantPool,
            }
        );
        let expanded = derive_class_parser_inner(&code)?;
        let raw_code = expanded.to_string();
        assert!(raw_code.contains("ctx . constant_pool ="));
        print_expanded_fmt(expanded);
        Ok(())
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
    fn test_get_match_arms_named_expand() -> Result<(), Box<dyn Error>> {
        let varaint: Variant = parse_quote!(A { b: B });
        let raw_code = generate_match_arms(&varaint)?;
        assert!(raw_code.contains("TestEnum :: A { b }"));
        println!("{}", raw_code);
        Ok(())
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
            #[class_parser(enum_entry(index(map[u8])))]
            enum TestEnum {
                A(a),
                B
            }
        };
        let expanded = resolve_enum(&code)?;
        let raw_code = expanded.to_string();
        assert!(raw_code.contains("let index = "));
        assert!(raw_code.contains("ContextIndex :: get (& ctx . map , index) ;"));
        print_expanded_fmt(expanded);
        Ok(())
    }
}
