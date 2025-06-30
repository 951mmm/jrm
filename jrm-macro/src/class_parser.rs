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
    let ItemStruct {
        fields,
        ident,
        attrs,
        ..
    } = item_struct;
    let result = match fields {
        Fields::Named(fields_named) => resolve_named(fields_named, ident)?,
        Fields::Unnamed(fields_unnamed) => resolve_unnamed(fields_unnamed, ident, attrs)?,
        _ => unreachable!(),
    };
    Ok(result)
}

fn resolve_enum(item_enum: &ItemEnum) -> syn::Result<proc_macro2::TokenStream> {
    let ItemEnum {
        variants, ident, ..
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

        let mut temp_idents = vec![];
        let mut parse_stmts = vec![];
        for (index, field) in fields.into_iter().enumerate() {
            let field_ty = &field.ty;

            let temp_ident = format_ident!("temp_{}", index);
            let stmt = quote! {
                let #temp_ident = <#field_ty as ClassParser>::parse(class_reader)?;
            };
            temp_idents.push(temp_ident);
            parse_stmts.push(stmt);
        }

        let discr_val_lit = Lit::new(Literal::i32_unsuffixed(discr_val));

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
    Ok(quote! {
        impl ClassLookUpParser for #ident {
            fn parse(class_reader: &mut ClassReader, prev: usize) -> anyhow::Result<Self> {
                let result = match prev {
                    #(#arm_expr)*
                    _ => {
                        unreachable!()
                    }
                };
                Ok(result)
            }
        }
    })
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

        let is_not_zero = attr_not_zero(field);
        let is_impl_sized = attr_impl_sized(field);
        let lookup_ident = attr_with_lookup(field)?;

        let stmt = if let Some(lookup_ident) = lookup_ident {
            quote! {
                let #field_ident = <#field_ty as ClassLookUpParser>::parse(class_reader, #lookup_ident as usize)?;
            }
        } else if is_not_zero {
            quote! {
                let #field_ident = <#field_ty as ClassParser>::parse(class_reader)?;
                if #field_ident == 0 {
                    return anyhow::bail!("invalid {}", stringify!(#field_ty));
                }
            }
        } else {
            quote! {
                let #field_ident = <#field_ty as ClassParser>::parse(class_reader)?;
            }
        };

        if is_impl_sized {
            collection_impl_blocks.push(resolve_collection_impl(field_ty, false)?);
        }
        parse_stmts.push(stmt);
        field_idents.push(field_ident);
    }
    Ok(quote! {
        impl ClassParser for #struct_ident {
            fn parse(class_reader: &mut ClassReader) -> anyhow::Result<Self> {
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
    struct_attrs: &Vec<Attribute>,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut temp_idents = vec![];
    let mut parse_stmts = vec![];
    let mut collection_impl_block = None;
    let is_sized_wrapper = attr_sized_wrapper(struct_attrs);

    for (index, field) in (&fields_unnamed.unnamed).into_iter().enumerate() {
        let field_ty = &field.ty;
        let temp_ident = format_ident!("temp_{}", index);

        let is_not_zero = attr_not_zero(field);
        let is_lookup_outer = attr_lookup_outer(field);
        let is_impl_sized = attr_impl_sized(field);
        let is_constant_pool = attr_constant_pool(field);
        let stmt = if is_not_zero {
            quote! {
                let #temp_ident = <#field_ty as ClassParser>::parse(class_reader)?;
                if #temp_ident == 0 {
                    return anyhow::bail!("invalid {}", stringify!(#field_ty));
                }
            }
        } else if is_lookup_outer {
            quote! {
                let #temp_ident = <#field_ty as ClassLookUpParser>::parse(class_reader, prev)?;
            }
        } else {
            quote! {
                let #temp_ident = <#field_ty as ClassParser>::parse(class_reader)?;
            }
        };

        if is_impl_sized {
            collection_impl_block = Some(resolve_collection_impl(field_ty, is_constant_pool)?);
        }
        temp_idents.push(temp_ident);
        parse_stmts.push(stmt);
    }
    let result = if is_sized_wrapper {
        quote! {
            impl ClassLookUpParser for #struct_ident {
                fn parse(class_reader: &mut ClassReader, prev: usize) -> anyhow::Result<Self> {
                    #(#parse_stmts)*
                    Ok(#struct_ident(#(#temp_idents),*))
                }
            }
            #collection_impl_block
        }
    } else {
        quote! {
            impl ClassParser for #struct_ident {
                fn parse(class_reader: &mut ClassReader) -> anyhow::Result<Self> {
                    #(#parse_stmts)*
                    Ok(#struct_ident(#(#temp_idents),*))
                }
            }
            #collection_impl_block
        }
    };
    Ok(result)
}

fn attr_not_zero(field: &Field) -> bool {
    field
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("not_zero"))
}
fn attr_impl_sized(field: &Field) -> bool {
    field
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("impl_sized"))
}
fn attr_with_lookup(field: &Field) -> syn::Result<Option<Ident>> {
    for attr in &field.attrs {
        if attr.path().is_ident("with_lookup") {
            return Ok(Some(attr.parse_args::<Ident>()?));
        }
    }
    Ok(None)
}
fn attr_lookup_outer(field: &Field) -> bool {
    field
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("lookup_outer"))
}
fn attr_sized_wrapper(attrs: &Vec<Attribute>) -> bool {
    attrs
        .iter()
        .any(|attr| attr.path().is_ident("sized_wrapper"))
}
fn attr_constant_pool(field: &Field) -> bool {
    field
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("constant_pool"))
}
fn resolve_collection_impl(
    collection_ty: &Type,
    is_constant_pool: bool,
) -> syn::Result<proc_macro2::TokenStream> {
    let collection_ident = get_collection_ident(collection_ty)?;
    let inner_ty = get_inner_ty(collection_ty)?;
    let stmts = if is_constant_pool {
        quote! {
            let mut collection = #collection_ident::with_capacity(prev);
            let invalid = ConstantWrapper { tag: 0, constant: Constant::Invalid };
            collection.push(invalid);
            for _ in 0..prev-1 {
                let item = <#inner_ty as ClassParser>::parse(class_reader)?;
                collection.push(item);
            }
            Ok(collection)
        }
    } else {
        quote! {
            let mut collection = #collection_ident::with_capacity(prev);
            for _ in 0..prev {
                let item = <#inner_ty as ClassParser>::parse(class_reader)?;
                collection.push(item);
            }
            Ok(collection)
        }
    };
    Ok(quote! {
        impl ClassLookUpParser for #collection_ty {
            fn parse(class_reader: &mut ClassReader, prev: usize) -> anyhow::Result<Self> {
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
