use base_macro::simple_field_attr;
use proc_macro2::Literal;
use quote::{format_ident, quote};
use syn::{
    Attribute, Field, Fields, FieldsNamed, FieldsUnnamed, GenericArgument, Ident, Item, ItemEnum,
    ItemStruct, Lit, LitInt, PathArguments, Type, TypePath, parse_quote,
};

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
    let mut arm_expr = vec![];
    let mut cur_discr = 0;
    for variant in variants {
        let variant_ident = &variant.ident;
        let fields = &variant.fields;
        let discr_val = match &variant.discriminant {
            Some((_, expr)) => {
                let lit_int: LitInt = parse_quote!(#expr);
                let val: i32 = lit_int.base10_parse()?;
                cur_discr = val + 1;
                val
            }
            None => {
                let val = cur_discr;
                cur_discr += 1;
                val
            }
        };
        let discr_val_lit = Lit::new(Literal::i32_unsuffixed(discr_val));

        let mut temp_idents = vec![];
        let mut parse_stmts = vec![];
        for (index, field) in fields.into_iter().enumerate() {
            let field_ty = &field.ty;

            let temp_ident = format_ident!("temp_{}", index);
            let stmt = quote! {
                let #temp_ident = <#field_ty as ClassParser>::parse(class_reader, ctx)?;
            };
            temp_idents.push(temp_ident);
            parse_stmts.push(stmt);
        }

        let expr = if fields.is_empty() {
            quote! {
                #discr_val_lit => {
                    #ident::#variant_ident
                }
            }
        } else {
            quote! {
                #discr_val_lit => {
                    #(#parse_stmts)*
                    #ident::#variant_ident(#(#temp_idents),*)
                }
            }
        };
        arm_expr.push(expr);
    }

    let result = quote! {
        impl ClassParser for #ident {
            fn parse(class_reader: &mut ClassReader, ctx: &mut ParserContext) -> anyhow::Result<Self> {
                let choice = ctx.count.clone();
                let result = match choice {
                    #(#arm_expr)*
                    _ => {
                        unreachable!()
                    }
                };
                return Ok(result);

            }
        }
    };
    Ok(result)
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

        let is_impl_sized = attr_impl_sized(field);
        let is_set_count = attr_set_count(field);
        let is_constant_index_end = attr_constant_index_end(field);
        let is_constant_index_check = attr_constant_index_check(field);

        let stmt = quote! {
            let #field_ident = <#field_ty as ClassParser>::parse(class_reader, ctx)?;
        };

        if is_impl_sized {
            collection_impl_blocks.push(resolve_collection_impl(field_ty, false)?);
        }
        parse_stmts.push(stmt);
        if is_set_count {
            parse_stmts.push(quote! {
                ctx.count = #field_ident as usize;
            });
        }
        if is_constant_index_end {
            parse_stmts.push(quote! {
                ctx.constant_index_range = 1..#field_ident;
            });
        }
        if is_constant_index_check {
            parse_stmts.push(quote! {
                if !ctx.constant_index_range.contains(&#field_ident) {
                    anyhow::bail!("invalid {}, not in range {:?}", stringify!(#field_ident), ctx.constant_index_range);
                }
            });
        }
        field_idents.push(field_ident);
    }

    Ok(quote! {
        impl ClassParser for #struct_ident {
            fn parse(class_reader: &mut ClassReader, ctx: &mut ParserContext) -> anyhow::Result<Self> {
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

        let is_impl_sized = attr_impl_sized(field);
        let is_constant_pool = attr_constant_pool(field);
        let stmt = quote! {
            let #temp_ident = <#field_ty as ClassParser>::parse(class_reader, ctx)?;
        };

        if is_impl_sized {
            collection_impl_block = Some(resolve_collection_impl(field_ty, is_constant_pool)?);
        }
        parse_stmts.push(stmt);
        if is_constant_pool {
            parse_stmts.push(quote! {
                ctx.constant_pool = #temp_ident.clone();
            });
        }
        temp_idents.push(temp_ident);
    }
    let result = quote! {
        impl ClassParser for #struct_ident {
            fn parse(class_reader: &mut ClassReader, ctx: &mut ParserContext) -> anyhow::Result<Self> {
                #(#parse_stmts)*
                Ok(#struct_ident(#(#temp_idents),*))
            }
        }
        #collection_impl_block
    };
    Ok(result)
}

// fn attr_not_zero(field: &Field) -> bool {
//     simple_attr!(field, "not_zero")
// }

macro_rules! paren_attr {
    ($field: ident, $name: literal, $ty: ty) => {
        for attr in &$field.attrs {
            if attr.path().is_ident($name) {
                return Ok(Some(attr.parse_args::<$ty>()?));
            }
        }
    };
}
// fn attr_impl_sized(field: &Field) -> syn::Result<Option<Ident>> {
//     paren_attr!(field, "impl_sized", Ident);
//     Ok(None)
// }
simple_field_attr! {"impl_sized"}
simple_field_attr! {"set_count"}
simple_field_attr! {"get_count"}
simple_field_attr! {"constant_pool"}
simple_field_attr! {"constant_index_end"}
simple_field_attr! {"constant_index_check"}
#[allow(unused_variables)]
fn resolve_collection_impl(
    collection_ty: &Type,
    is_constant_pool: bool,
) -> syn::Result<proc_macro2::TokenStream> {
    let collection_ident = get_collection_ident(collection_ty)?;
    let inner_ty = get_inner_ty(collection_ty)?;
    let stmts = if is_constant_pool {
        quote! {
            let mut collection = #collection_ident::with_capacity(size);
            let invalid = ConstantWrapper { tag: 0, constant: Constant::Invalid };
            collection.push(invalid);
            for _ in 0..size-1 {
                let item = <#inner_ty as ClassParser>::parse(class_reader, ctx)?;
                collection.push(item);
            }
            return Ok(collection);

        }
    } else {
        quote! {
            let mut collection = #collection_ident::with_capacity(size);
            for _ in 0..size {
                let item = <#inner_ty as ClassParser>::parse(class_reader, ctx)?;
                collection.push(item);
            }
            return Ok(collection);
        }
    };
    Ok(quote! {
        impl ClassParser for #collection_ty {
            fn parse(class_reader: &mut ClassReader, ctx: &mut ParserContext) -> anyhow::Result<Self> {
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
    Err(syn::Error::new_spanned(ty, "failed to get collection ty"))
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
    Err(syn::Error::new_spanned(ty, "Invalid inner type for Vec"))
}
