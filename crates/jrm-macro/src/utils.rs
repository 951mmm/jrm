use quote::ToTokens;
use syn::{Type, TypePath};

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
