use macro_utils::{FromAttrs, syn_err};
use syn::{Attribute, Ident, Token, Type, bracketed, parenthesized, parse::Parse};

use crate::utils::try_extract_attr_meta_list;

#[derive(Debug)]
pub enum EnumEntry {
    Get,
    Index(Box<IndexMeta>),
    Set,
    None,
}

///
/// #[enum_entry(index(map=map[ty], outer))]
/// 是有序对
#[derive(Debug)]
pub struct IndexMeta {
    pub index_ty: Type,
    pub map_ident: Ident,
    pub outer: bool,
}

impl Parse for IndexMeta {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        if ident != "map" {
            syn_err!("invalid index meta, `map` is required");
        }
        input.parse::<Token![=]>()?;
        let ident: Ident = input.parse()?;
        let map_ident = ident;

        let content;
        bracketed!(content in input);
        let ty: Type = content.parse()?;
        let index_ty = ty;
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let ident: Ident = input.parse()?;
            if ident == "outer" {
                return Ok(Self {
                    index_ty,
                    map_ident,
                    outer: true,
                });
            } else {
                syn_err!(
                    "invalid index meta, found: {}, consider `index(map = ..., outer)`",
                    ident
                );
            }
        }
        Ok(Self {
            index_ty,
            map_ident,
            outer: false,
        })
    }
}

impl FromAttrs for EnumEntry {
    fn from_attrs(attrs: &[Attribute]) -> syn::Result<Self>
    where
        Self: Sized,
    {
        let mut enum_entry = EnumEntry::None;
        let meta_list = try_extract_attr_meta_list(attrs, "class_parser", "enum_entry")?;
        if meta_list.is_none() {
            return Ok(enum_entry);
        }
        meta_list.unwrap().parse_nested_meta(|meta| {
            // #[enum_entry(get)]
            if meta.path.is_ident("get") {
                enum_entry = EnumEntry::Get;
                return Ok(());
            }
            // #[enum_entry(index(map[ty]))]
            if meta.path.is_ident("index") {
                let content;
                parenthesized!(content in meta.input);
                let index_meta: IndexMeta = content.parse()?;
                enum_entry = EnumEntry::Index(Box::new(index_meta));
                return Ok(());
            }
            if meta.path.is_ident("set") {
                enum_entry = EnumEntry::Set;
                return Ok(());
            }
            Err(meta.error(format!(
                "expected `get`, `index`, `set`, found: {}",
                meta.input,
            )))
        })?;
        Ok(enum_entry)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use quote::format_ident;
    use rstest::rstest;
    use syn::{Attribute, parse_quote};

    use crate::class_parser::enum_entry::{EnumEntry, IndexMeta};
    use macro_utils::FromAttrs;

    #[rstest]
    #[case(parse_quote!(#[class_parser(enum_entry(get))]), EnumEntry::Get, "enum entry get")]
    #[case(parse_quote!(#[class_parser(enum_entry(set))]), EnumEntry::Set, "enum entry set")]
    #[case(parse_quote!(#[class_parser(enum_entry(index(map = map[u8])))]), EnumEntry::Index(Box::new(IndexMeta {
        map_ident: format_ident!("map"),
        index_ty: parse_quote!(u8),
        outer: false
    })), "enun entry inner")]
    #[case(parse_quote!(#[class_parser(enum_entry(index(map = map[u8], outer)))]), EnumEntry::Index(Box::new(IndexMeta {
        map_ident: format_ident!("map"),
        index_ty: parse_quote!(u8),
        outer: true
    })), "enum entry outer")]
    fn test_attr_enum_entry(
        #[case] input: Attribute,
        #[case] expected: EnumEntry,
        #[case] desc: &str,
    ) -> Result<(), Box<dyn Error>> {
        use macro_utils::FromAttrs;

        let output = EnumEntry::from_attrs(&[input])?;
        assert_eq!(format!("{:?}", output), format!("{:?}", expected), "{desc}",);
        Ok(())
    }

    #[rstest]
    #[case(parse_quote!(#[class_parser(enum_entry(some))]), "first item is not `map`")]
    #[case(parse_quote!(#[class_parser(enum_entry(index(map = map[u8], inner)))]), "second item is not `outer`")]
    #[case(parse_quote!(#[class_parser(enum_entry(index(map = map[u8], outer, other)))]), "more than 2 item")]
    fn test_invalid_attr_enum_entry(#[case] input: Attribute, #[case] desc: &str) {
        assert!(EnumEntry::from_attrs(&[input]).is_err(), "{desc}");
    }
}
