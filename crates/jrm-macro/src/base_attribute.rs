use base_macro::syn_err;
use convert_case::{Case, Casing};
use darling::{FromMeta, util::Flag};
use quote::{format_ident, quote};
use syn::{
    Field, Fields, Ident, ItemStruct, Token, Type, TypePath, parse_quote, punctuated::Punctuated,
};

use crate::utils::try_extract_outer_ty_string;
#[derive(Debug, FromMeta)]
#[darling(derive_syn_parse)]
pub struct Attrs {
    #[darling(default)]
    suffix: Option<AttrSuffix>,
    #[darling(default)]
    single: Option<AttrSingle>,
    #[darling(default)]
    impled: Flag,
}

#[derive(Debug, FromMeta)]
struct AttrSuffix {
    count_ident: Ident,
    #[darling(default)]
    item_ty: Option<TypePath>,
    #[darling(default)]
    rename: Option<Ident>,
}

#[derive(Debug, FromMeta)]
struct AttrSingle {
    ident: Ident,
    ty: TypePath,
    #[darling(default)]
    constant_index_check: Flag,
}

pub fn base_attrubute_inner(
    attrs: &Attrs,
    item_struct: &mut ItemStruct,
) -> syn::Result<proc_macro2::TokenStream> {
    let index_field_prefix: Field =
        parse_quote! {#[enum_entry(get)] #[constant_index(check)] pub attribute_name_index: u16};
    let length_field_prefix: Field = parse_quote! {pub attribute_length: u32};
    if let Fields::Named(ref mut field_named) = item_struct.fields {
        let mut new_named: Punctuated<Field, Token![,]> = Punctuated::new();
        new_named.push(index_field_prefix);
        let is_impled = attrs.impled.is_present();
        if let Some(AttrSuffix {
            count_ident,
            item_ty,
            rename,
        }) = &attrs.suffix
        {
            new_named.push(length_field_prefix);
            new_named.extend(field_named.named.clone());
            new_named.push(parse_quote!(#[count(set)] #[getter(skip)] #count_ident: u16));

            let list_ident = get_suffix_list_ident(is_impled, rename.clone(), item_ty)?;
            let list_field = get_suffix_list_field(is_impled, &list_ident, item_ty)?;
            new_named.push(parse_quote!(#list_field));
        } else if let Some(AttrSingle {
            ident,
            ty,
            constant_index_check,
        }) = &attrs.single
        {
            let is_collection_ty = is_collection_ty(ty);
            let ty = build_ty_from(ty);
            let length_field_prefix = if is_collection_ty {
                parse_quote!(
                    #[count(set)]
                    #length_field_prefix
                )
            } else {
                length_field_prefix
            };
            new_named.push(length_field_prefix);
            let single_suffix_field = if constant_index_check.is_present() {
                parse_quote!(
                    #[constant_index(check)]
                    #ident: #ty
                )
            } else if is_collection_ty {
                if is_impled {
                    parse_quote!(
                        #[count(impled)]
                        #ident: #ty
                    )
                } else {
                    parse_quote!(
                        #[count(get)]
                        #ident: #ty
                    )
                }
            } else {
                parse_quote!(
                    #ident: #ty
                )
            };
            new_named.push(single_suffix_field);
        } else {
            new_named.push(length_field_prefix);
        }
        field_named.named = new_named;
    }
    Ok(quote! {
        #item_struct
    })
}

fn get_suffix_list_ident(
    is_impled: bool,
    rename: Option<Ident>,
    item_ty: &Option<TypePath>,
) -> syn::Result<Ident> {
    let result = match rename {
        Some(rename_ident) => rename_ident,
        None => {
            if is_impled {
                match item_ty {
                    Some(item_ty) => {
                        let outer_ty_string = try_extract_outer_ty_string(&build_ty_from(item_ty))?;
                        let outer_ty_string =
                            outer_ty_string.from_case(Case::Camel).to_case(Case::Snake);
                        format_ident!("{}s", outer_ty_string)
                    }
                    None => parse_quote!(bytes),
                }
            } else {
                if item_ty.is_none() {
                    syn_err!("invalid attr, list type is required without `impled` attr");
                }

                let outer_ty_string =
                    try_extract_outer_ty_string(&build_ty_from(item_ty.as_ref().unwrap()))?;
                let outer_ty_string = outer_ty_string.from_case(Case::Camel).to_case(Case::Snake);
                format_ident!("{}s", outer_ty_string)
            }
        }
    };
    Ok(result)
}
fn get_suffix_list_field(
    is_impled: bool,
    list_ident: &Ident,
    item_ty: &Option<TypePath>,
) -> syn::Result<Field> {
    let result = if is_impled {
        match item_ty {
            Some(item_ty) => {
                parse_quote!(
                    #[count(impled)]
                    #list_ident: Vec<#item_ty>
                )
            }
            None => {
                parse_quote!(
                    #[count(impled)]
                    #list_ident: Vec<u8>
                )
            }
        }
    } else {
        if item_ty.is_none() {
            syn_err!("invalid attr, list type is required without `impled` attr");
        }
        let item_ty = item_ty.as_ref().unwrap();
        parse_quote!(
            #[count(get)]
            #list_ident: Vec<#item_ty>
        )
    };
    Ok(result)
}

fn build_ty_from(ty_path: &TypePath) -> Type {
    Type::Path(ty_path.clone())
}

fn is_collection_ty(ty: &TypePath) -> bool {
    if let Some(segment) = ty.path.segments.last() {
        if segment.ident == "Vec" {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use macro_utils::print_expanded_fmt;
    use quote::ToTokens;
    use syn::{Field, ItemStruct, TypePath, parse_quote};

    use crate::{
        base_attribute::{
            AttrSuffix, Attrs, build_ty_from, get_suffix_list_field, get_suffix_list_ident,
            is_collection_ty, try_extract_outer_ty_string,
        },
        base_attrubute_inner,
    };
    #[test]
    fn test_is_collection_ty() {
        let ty: TypePath = parse_quote!(Vec<i32>);
        assert!(is_collection_ty(&ty));

        let ty: TypePath = parse_quote!(Attr<Other>);
        assert!(!is_collection_ty(&ty));
    }
    #[test]
    fn test_get_suffix_list_ident() -> Result<(), Box<dyn Error>> {
        let is_impled = false;
        let rename = Some(parse_quote!(some_ident));
        let item_ty = Some(parse_quote!(SomeType));
        let list_ident = get_suffix_list_ident(is_impled, rename, &item_ty)?;
        assert_eq!(list_ident, "some_ident");

        let rename = None;
        let list_ident = get_suffix_list_ident(is_impled, rename.clone(), &item_ty)?;
        assert_eq!(list_ident, "some_types");

        let item_ty = None;
        let err = get_suffix_list_ident(is_impled, rename, &item_ty)
            .err()
            .unwrap();
        assert_eq!(
            err.to_string(),
            "invalid attr, list type is required without `impled` attr"
        );

        let is_impled = true;
        let rename = Some(parse_quote!(some_ident));
        let list_ident = get_suffix_list_ident(is_impled, rename, &item_ty)?;
        assert_eq!(list_ident, "some_ident");

        let rename = None;
        let list_ident = get_suffix_list_ident(is_impled, rename.clone(), &item_ty)?;
        assert_eq!(list_ident, "bytes");

        let item_ty = Some(parse_quote!(SomeType));
        let list_ident = get_suffix_list_ident(is_impled, rename, &item_ty)?;
        assert_eq!(list_ident, "some_types");

        Ok(())
    }

    #[test]
    fn test_get_suffix_list_field() -> Result<(), Box<dyn Error>> {
        let mut results = [
            "# [count (impled)] some_ident : Vec < SomeType >",
            "# [count (impled)] some_ident : Vec < u8 >",
            "# [count (get)] some_ident : Vec < SomeType >",
            "# [count (get)] some_ident : Vec < Attribute >",
        ]
        .iter()
        .map(ToString::to_string);
        // let mut results = results.into_iter();

        let is_impled = true;
        let field_ident = parse_quote!(some_ident);
        let item_ty = Some(parse_quote!(SomeType));
        let field = get_suffix_list_field(is_impled, &field_ident, &item_ty)?;

        assert_eq!(field_string(field), results.next().unwrap());

        let item_ty = None;
        let field = get_suffix_list_field(is_impled, &field_ident, &item_ty)?;
        assert_eq!(field_string(field), results.next().unwrap());

        let is_impled = false;
        let item_ty = Some(parse_quote!(SomeType));
        let field = get_suffix_list_field(is_impled, &field_ident, &item_ty)?;
        assert_eq!(field_string(field), results.next().unwrap());

        let item_ty = Some(parse_quote!(Attribute));
        let field = get_suffix_list_field(is_impled, &field_ident, &item_ty)?;
        assert_eq!(field_string(field), results.next().unwrap());

        let item_ty = None;
        let err = get_suffix_list_field(is_impled, &field_ident, &item_ty)
            .err()
            .unwrap();
        assert_eq!(
            err.to_string(),
            "invalid attr, list type is required without `impled` attr"
        );

        Ok(())
    }
    fn field_string(field: Field) -> String {
        format!("{}", field.to_token_stream())
    }
    #[test]
    fn test_attrs_parse() -> Result<(), Box<dyn Error>> {
        let attrs: Attrs = parse_quote!(suffix(count_ident = count, item_ty = SomeAttribute));
        let AttrSuffix {
            count_ident,
            item_ty,
            rename,
        } = attrs.suffix.unwrap();
        assert_eq!(count_ident, "count");
        let ty = build_ty_from(item_ty.as_ref().unwrap());
        assert_eq!(try_extract_outer_ty_string(&ty)?, "SomeAttribute");
        assert!(rename.is_none());

        let attrs: Attrs = parse_quote!(suffix(
            count_ident = count,
            item_ty = SomeType,
            rename = other_ident
        ));
        let rename = attrs.suffix.unwrap().rename.unwrap();
        assert_eq!(rename, "other_ident");

        let attrs: Attrs = parse_quote!(suffix(count_ident = count, item_ty = SomeType), impled);
        assert!(attrs.impled.is_present());
        Ok(())
    }
    #[test]
    fn test_base_attribute_suffix_expand() -> Result<(), Box<dyn Error>> {
        let attrs: Attrs = parse_quote!(suffix(count_ident = count, item_ty = SomeAttribute));
        let (raw_code, expanded) = base_attribute_expand(&attrs)?;
        assert!(raw_code.contains("# [enum_entry (get)]"));
        assert!(raw_code.contains("count : u16"));
        assert!(raw_code.contains("pub attribute_name_index : u16"));
        assert!(raw_code.contains("Vec < SomeAttribute >"));
        assert!(raw_code.contains("# [count (get)]"));
        print_expanded_fmt(expanded);
        Ok(())
    }
    #[test]
    fn test_base_attribute_single_expand() -> Result<(), Box<dyn Error>> {
        let attrs: Attrs = parse_quote!(single(ident = some, ty = Some, constant_index_check));
        let (raw_code, expanded) = base_attribute_expand(&attrs)?;
        assert!(raw_code.contains("# [constant_index (check)] some : Some"));
        println!("#1");
        print_expanded_fmt(expanded);

        let attrs: Attrs = parse_quote!(single(ident = some, ty = "Vec<Some>"));
        let (raw_code, expanded) = base_attribute_expand(&attrs)?;
        assert!(raw_code.contains("# [count (set)] pub attribute_length"));
        assert!(raw_code.contains("# [count (get)]"));
        println!("#2");
        print_expanded_fmt(expanded);

        let attrs: Attrs = parse_quote!(single(ident = some, ty = "Vec<Some>"), impled);
        let (raw_code, expanded) = base_attribute_expand(&attrs)?;
        assert!(raw_code.contains("# [count (impled)]"));
        println!("#3");
        print_expanded_fmt(expanded);
        Ok(())
    }
    fn generate_struct() -> ItemStruct {
        parse_quote!(
            struct StructTest {
                a: i32,
                b: u8,
            }
        )
    }

    fn base_attribute_expand(attrs: &Attrs) -> syn::Result<(String, proc_macro2::TokenStream)> {
        let mut __struct = generate_struct();
        let expanded = base_attrubute_inner(attrs, &mut __struct)?;
        let raw_code = expanded.to_string();
        Ok((raw_code, expanded))
    }
}
