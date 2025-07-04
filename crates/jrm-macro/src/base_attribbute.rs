use convert_case::{Case, Casing};
use quote::{ToTokens, format_ident, quote};
use syn::{
    Field, Fields, Ident, ItemStruct, Token, Type, TypePath, parenthesized, parse::Parse,
    parse_quote, punctuated::Punctuated,
};
pub struct Attrs {
    suffix: Option<(Ident, Type)>,
}
impl Parse for Attrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut suffix = None;

        let attrs = Punctuated::<Attr, Token![,]>::parse_terminated(input)?;
        for attr in attrs {
            match attr {
                Attr::Suffix {
                    count_ident,
                    list_ty,
                } => suffix = Some((count_ident, list_ty)),
            }
        }
        Ok(Attrs { suffix })
    }
}
enum Attr {
    Suffix { count_ident: Ident, list_ty: Type },
}
impl Parse for Attr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attr_ident: &Ident = &input.parse()?;
        let content;
        parenthesized!(content in input);

        match attr_ident.to_string().as_str() {
            "suffix" => {
                let count_ident: Ident = content.parse()?;
                content.parse::<Token![,]>()?;
                let list_ty: Type = content.parse()?;
                Ok(Attr::Suffix {
                    count_ident,
                    list_ty,
                })
            }
            _ => Err(syn::Error::new_spanned(
                attr_ident,
                format_args!("unsurpport attr: {}", attr_ident),
            )),
        }
    }
}

pub fn base_attrubute_inner(
    attrs: &Attrs,
    item_struct: &mut ItemStruct,
) -> syn::Result<proc_macro2::TokenStream> {
    let base_fields_prefix = [
        quote! {attribute_name_index: u16},
        quote! {attribute_length: u32},
    ];
    if let Fields::Named(ref mut field_named) = item_struct.fields {
        let mut new_named: Punctuated<Field, Token![,]> = Punctuated::new();
        for base_field in base_fields_prefix {
            new_named.push(parse_quote!(#base_field));
        }
        new_named.extend(field_named.named.clone());
        if let Some((count_ident, list_ty)) = &attrs.suffix {
            let list_ty_snake = get_ty_string(list_ty)?
                .from_case(Case::Camel)
                .to_case(Case::Snake);
            let list_ident = format_ident!("{}s", list_ty_snake);
            new_named.push(parse_quote!(
                #count_ident: u16
            ));
            new_named.push(parse_quote!(
                #list_ident: Vec<#list_ty>
            ));
        }
        field_named.named = new_named;
    }
    Ok(quote! {
        #item_struct
    })
}
fn get_ty_string(ty: &Type) -> syn::Result<String> {
    let err = Err(syn::Error::new_spanned(
        ty,
        format_args!("failed to parse ty: {}", ty.to_token_stream()),
    ));
    match ty {
        Type::Path(TypePath { path, .. }) => {
            if let Some(segment) = path.segments.last() {
                Ok(segment.ident.to_string())
            } else {
                err
            }
        }
        _ => err,
    }
}
