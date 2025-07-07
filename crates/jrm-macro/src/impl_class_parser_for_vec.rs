use quote::quote;
use syn::Type;

use crate::utils::try_extract_outer_ty_string;

pub fn impl_class_parser_for_vec_inner(ty: &Type) -> syn::Result<proc_macro2::TokenStream> {
    let ty_lit = try_extract_outer_ty_string(ty)?;
    let result = if ty_lit == "u8" {
        quote! {
            impl ClassParser for Vec<#ty> {
                fn parse(ctx: &mut ParserContext) -> anyhow::Result<Self>
                where
                    Self: Sized,
                {
                    let size = ctx.count;
                    let bytes = ctx.class_reader.read_bytes(size).unwrap_or_default();
                    Ok(bytes)
                }
            }
        }
    } else {
        quote! {
            impl ClassParser for Vec<#ty> {
                fn parse(ctx: &mut ParserContext) -> anyhow::Result<Self> {
                    let size = ctx.count.clone();
                    let mut collection = Vec::with_capacity(size);
                    for _ in 0..size {
                        let item = <#ty as ClassParser>::parse(ctx)?;
                        collection.push(item);
                    }
                    Ok(collection)
                }
            }
        }
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::impl_class_parser_for_vec_inner;
    use macro_utils::print_expanded_fmt;
    use std::error::Error;
    use syn::{Type, parse_quote};

    #[test]
    fn test_impl_class_parser_for_vec_expand() -> Result<(), Box<dyn Error>> {
        let code: Type = parse_quote!(SomeT);
        let expanded = impl_class_parser_for_vec_inner(&code)?;
        print_expanded_fmt(expanded);
        Ok(())
    }
    #[test]
    fn test_impl_class_parser_for_vec_u8_expand() -> Result<(), Box<dyn Error>> {
        let ty: Type = parse_quote!(u8);
        let expanded = impl_class_parser_for_vec_inner(&ty)?;
        let raw_code = expanded.to_string();
        assert!(raw_code.contains("read_bytes"));
        Ok(())
    }
}
