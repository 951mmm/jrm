use convert_case::{Case, Casing};
use macro_utils::{FromAttrs, syn_err};
use quote::{format_ident, quote};
use syn::{
    Field, Fields, FieldsNamed, FieldsUnnamed, ItemEnum, Lifetime,
    braced, parse::Parse, parse_quote,
};

pub fn derive_parse_variant_inner(item_enum: &ItemEnum) -> syn::Result<proc_macro2::TokenStream> {
    let enum_ident = &item_enum.ident;
    let mut parse_fns = vec![];
    let mut fields_named_variant_structs = vec![];

    for variant in &item_enum.variants {
        let variant_ident = &variant.ident;
        let variant_ident_snake = variant_ident
            .to_string()
            .from_case(Case::Camel)
            .to_case(Case::Snake);
        let fn_ident = format_ident!("parse_{}", variant_ident_snake);
        let Pass {
            pass_attrs: variant_pass_attrs,
            ..
        } = Pass::from_attrs(&variant.attrs)?;
        let parse_fn = match &variant.fields {
            Fields::Named(FieldsNamed { named: fields, .. }) => {
                let field_idents: Vec<_> = fields.iter().map(|field| &field.ident).collect();
                let life_time = Lifetime::new(
                    &format!("'{}", variant_ident_snake),
                    proc_macro2::Span::call_site(),
                );
                let mut struct_fields: Vec<Field> = fields.clone().into_iter().collect();
                for struct_field in struct_fields.iter_mut() {
                    let ty = &struct_field.ty;
                    struct_field.ty = parse_quote! {&#life_time #ty};

                    let Pass {
                        pass_attrs: field_pass_attrs,
                    } = Pass::from_attrs(&struct_field.attrs)?;
                    struct_field.attrs = parse_quote!(#field_pass_attrs);
                }
                fields_named_variant_structs.push(quote! {
                    #variant_pass_attrs
                    pub struct #variant_ident<#life_time> {
                        #(#struct_fields),*
                    }
                });
                quote! {
                    #[inline]
                    #[allow(unused)]
                    pub fn #fn_ident(&self) -> anyhow::Result<#variant_ident> {
                        if let #enum_ident::#variant_ident { #(#field_idents),* } = self {
                            return Ok(#variant_ident {
                                #(#field_idents),*
                            });
                        }
                        anyhow::bail!("failed to parse from {} to {}", stringify!(#enum_ident), stringify!(#variant_ident))
                    }
                }
            }
            Fields::Unnamed(FieldsUnnamed {
                unnamed: fields, ..
            }) => {
                if fields.len() > 1 {
                    syn_err!(fields, "invalid turple variant, expected 1 item");
                }
                let field = fields.first().unwrap();
                let field_ty = &field.ty;
                let temp_ident = format_ident!("temp");
                quote! {
                    #[inline]
                    #[allow(unused)]
                    pub fn #fn_ident(&self) -> anyhow::Result<&#field_ty> {
                        if let #enum_ident::#variant_ident(#temp_ident) = self {
                            return Ok(#temp_ident);
                        }
                        anyhow::bail!("failed to parse from {} to {}", stringify!(#enum_ident), stringify!(#field_ty))
                    }
                }
            }
            Fields::Unit => {
                continue;
            }
        };
        parse_fns.push(parse_fn);
    }
    Ok(quote! {
        impl #enum_ident {
            #(#parse_fns)*
        }
        #(#fields_named_variant_structs)*
    })
}

struct Pass {
    pass_attrs: proc_macro2::TokenStream,
}
impl FromAttrs for Pass {
    fn from_attrs(attrs: &[syn::Attribute]) -> syn::Result<Self>
    where
        Self: Sized,
    {
        let mut pass_attrs = quote! {};
        for attr in attrs {
            if attr.path().is_ident("parse_variant") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("pass") {
                        let content;
                        braced!(content in meta.input);
                        let token_stream: proc_macro2::TokenStream = content.parse()?;
                        pass_attrs.extend(token_stream);
                        Ok(())
                    } else {
                        Err(
                            meta.error(format!("invalid meta: {},expected `pass`", meta.input))
                        )
                    }
                })?;
            } 
        }
        Ok(Pass { pass_attrs,  })
    }
}

#[cfg(test)]
mod test {
    use macro_utils::print_expanded_fmt;
    use rstest::rstest;
    use syn::{ItemEnum, parse_quote};

    use crate::derive_parse_variant_inner;

    #[rstest]
    #[case(parse_quote!(enum Some {
        #[parse_variant(pass {#[derive(Debug)]})]
        A {
            #[parse_variant(pass {#[some_macro(any_attrs)]})]
            b: B
        }
    }), &[
            "fn parse_a (& self)",
            "struct A < 'a >",
            "# [derive (Debug)] pub s",
            "# [some_macro (any_attrs)] b",
            "return Ok (A { b })"],
        "named field")]
    #[case(parse_quote!(enum Some {
        A(B)
    }), &["return Ok (temp) ;"],
        "unnamed field")]
    fn test_parse_variant_expand(
        #[case] input: ItemEnum,
        #[case] contains: &[&str],
        #[case] desc: &str,
    ) {
        let output = derive_parse_variant_inner(&input).unwrap();
        print_expanded_fmt(output.clone());
        assert!(
            contains
                .iter()
                .all(|item| output.to_string().contains(item)),
            "{desc}",
        );
    }
    #[rstest]
    #[case(parse_quote!(
        enum Some {
            #[parse_variant(other)]
            A {
                b: B 
            }
        }),
        "invalid meta",
        "unimplmented attrs"

    )]
    fn test_invalid_parse_variant(
        #[case] input: ItemEnum,
        #[case] err_str: &str,
        #[case] desc: &str,
    ) {
        let output = derive_parse_variant_inner(&input);
        assert!(output.unwrap_err().to_string().contains(err_str), "{desc}");
    }
}
