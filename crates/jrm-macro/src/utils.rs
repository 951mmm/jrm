use quote::ToTokens;
use syn::{Attribute, MetaList, Type, TypePath};

/// 不提取泛型，只提取最外层
/// some::A<B> -> B
pub fn try_extract_outer_ty_string(ty: &Type) -> syn::Result<String> {
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
pub fn try_extract_attr_meta_list<'a>(
    attrs: &'a [Attribute],
    attr_str: &'a str,
    meta_list_str: &'a str,
) -> syn::Result<Option<MetaList>> {
    for attr in attrs {
        if attr.path().is_ident(attr_str) {
            let meta_list: MetaList = attr.parse_args()?;
            if meta_list.path.is_ident(meta_list_str) {
                return Ok(Some(meta_list));
            }
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use syn::{Type, parse_quote};

    use crate::utils::try_extract_outer_ty_string;

    #[test]
    fn test_try_extract_outer_ty_string() -> Result<(), Box<dyn Error>> {
        let ty1: Type = parse_quote!(Vec<u32>);
        let ty2: Type = parse_quote!(Attribute);
        let ty_lit1 = try_extract_outer_ty_string(&ty1)?;
        assert_eq!(ty_lit1, "Vec");
        let ty_lit2 = try_extract_outer_ty_string(&ty2)?;
        assert_eq!(ty_lit2, "Attribute");
        Ok(())
    }
}
