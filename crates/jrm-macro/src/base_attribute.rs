use base_macro::syn_err;
use convert_case::{Case, Casing};
use quote::{format_ident, quote};
use syn::{
    Field, Fields, Ident, ItemStruct, Token, Type, parenthesized, parse::Parse, parse_quote,
    punctuated::Punctuated,
};

use crate::utils::try_extract_outer_ty_string;
pub struct Attrs {
    suffix: Option<(Ident, Option<Type>, Option<Ident>)>,
    impled: bool,
}

impl Parse for Attrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut suffix = None;
        let mut bytes = false;

        let attrs = Punctuated::<Attr, Token![,]>::parse_terminated(input)?;
        for attr in attrs {
            match attr {
                Attr::Suffix {
                    count_ident,
                    item_ty,
                    rename,
                } => {
                    suffix = Some((count_ident, item_ty, rename));
                }
                Attr::Impled => bytes = true,
            }
        }

        Ok(Attrs {
            suffix,
            impled: bytes,
        })
    }
}
enum Attr {
    Suffix {
        count_ident: Ident,
        item_ty: Option<Type>,
        rename: Option<Ident>,
    },
    Impled,
}
impl Parse for Attr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attr_ident: &Ident = &input.parse()?;

        match attr_ident.to_string().as_str() {
            "suffix" => {
                let content;
                parenthesized!(content in input);
                let count_ident: Ident = content.parse()?;
                let mut item_ty = None;
                if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                    let ty: Type = content.parse()?;
                    item_ty = Some(ty);
                }
                let mut rename = None;
                if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                    let ident: Ident = content.parse()?;
                    if ident != "rename" {
                        return Err(
                            input.error("invalid attr, last attr should be `rename` or not")
                        );
                    }
                    let content_inner;
                    parenthesized!(content_inner in content);
                    let rename_ident: Ident = content_inner.parse()?;
                    rename = Some(rename_ident);
                }
                Ok(Attr::Suffix {
                    count_ident,
                    item_ty,
                    rename,
                })
            }
            "impled" => Ok(Attr::Impled),
            _ => Err(input.error(format_args!("unsurpport attr: {}", attr_ident))),
        }
    }
}

pub fn base_attrubute_inner(
    attrs: &Attrs,
    item_struct: &mut ItemStruct,
) -> syn::Result<proc_macro2::TokenStream> {
    let base_fields_prefix = [
        quote! {#[enum_entry(get)] pub attribute_name_index: u16},
        quote! {pub attribute_length: u32},
    ];
    if let Fields::Named(ref mut field_named) = item_struct.fields {
        let mut new_named: Punctuated<Field, Token![,]> = Punctuated::new();
        for base_field in base_fields_prefix {
            new_named.push(parse_quote!(#base_field));
        }
        new_named.extend(field_named.named.clone());
        if let Some((count_ident, item_ty, rename)) = &attrs.suffix {
            new_named.push(parse_quote!(#[count(set)] #count_ident: u16));

            let is_impled = attrs.impled;
            let list_ident = get_suffix_list_ident(is_impled, rename.clone(), item_ty)?;
            let list_field = get_suffix_list_field(is_impled, &list_ident, item_ty)?;
            new_named.push(parse_quote!(#list_field));
            field_named.named = new_named;
        }
    }

    Ok(quote! {
        #item_struct
    })
}

fn get_suffix_list_ident(
    is_impled: bool,
    rename: Option<Ident>,
    item_ty: &Option<Type>,
) -> syn::Result<Ident> {
    let result = match rename {
        Some(rename_ident) => rename_ident,
        None => {
            if is_impled {
                match item_ty {
                    Some(item_ty) => {
                        let outer_ty_string = try_extract_outer_ty_string(item_ty)?;
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
                let outer_ty_string = try_extract_outer_ty_string(item_ty.as_ref().unwrap())?;
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
    item_ty: &Option<Type>,
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
// /**
//  * 判断列表元素的乐星是否为Attribute
//  */
// fn is_ty_lit_attribute(ty_lit: &str) -> bool {
//     if ty_lit == "Attribute" {
//         return true;
//     }
//     false
// }
#[cfg(test)]
mod tests {
    use std::error::Error;

    use macro_utils::print_expanded_fmt;
    use quote::ToTokens;
    use syn::{Field, ItemStruct, parse_quote};

    use crate::{
        base_attribute::{
            Attrs, get_suffix_list_field, get_suffix_list_ident, try_extract_outer_ty_string,
        },
        base_attrubute_inner,
    };

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
        let results: Vec<_> = [
            "# [count (impled)] some_ident : Vec < SomeType >",
            "# [count (impled)] some_ident : Vec < u8 >",
            "# [count (get)] some_ident : Vec < SomeType >",
            "# [count (get)] some_ident : Vec < Attribute >",
        ]
        .iter()
        .map(|str| str.to_string())
        .collect();
        let mut results = results.into_iter();

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
        let attrs: Attrs = parse_quote!(suffix(count, SomeAttribute));
        let (count_ident, item_ty, rename) = attrs.suffix.unwrap();
        assert_eq!(count_ident, "count");
        assert_eq!(
            try_extract_outer_ty_string(&item_ty.unwrap())?,
            "SomeAttribute"
        );
        assert!(rename.is_none());

        let attrs: Attrs = parse_quote!(suffix(count, SomeAttribute, rename(other_ident)));
        let rename = attrs.suffix.unwrap().2.unwrap();
        assert_eq!(rename, "other_ident");

        let attrs: Attrs = parse_quote!(suffix(count, SomeType), impled);
        assert!(attrs.impled);
        Ok(())
    }
    #[test]
    fn test_base_attribute_expand() -> Result<(), Box<dyn Error>> {
        let attrs = generate_attrs();
        let mut __struct = generate_struct();
        let expanded = base_attrubute_inner(&attrs, &mut __struct)?;
        let raw_code = expanded.to_string();
        assert!(raw_code.contains("# [enum_entry (get)]"));
        assert!(raw_code.contains("count : u16"));
        assert!(raw_code.contains("pub attribute_name_index : u16"));
        assert!(raw_code.contains("Vec < SomeAttribute >"));
        assert!(raw_code.contains("# [count (get)]"));
        print_expanded_fmt(expanded);
        Ok(())
    }
    fn generate_attrs() -> Attrs {
        parse_quote!(suffix(count, SomeAttribute))
    }
    fn generate_struct() -> ItemStruct {
        parse_quote!(
            struct StructTest {
                a: i32,
                b: u8,
            }
        )
    }
}
