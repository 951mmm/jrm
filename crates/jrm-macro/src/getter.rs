use base_macro::syn_err;
use darling::{FromField, util::Flag};
use quote::{format_ident, quote};
use syn::{Data, DataStruct, DeriveInput, Fields, FieldsNamed};

#[derive(FromField)]
#[darling(attributes(getter))]
struct FieldArgs {
    skip: Flag,
    copy: Flag,
    rename: Option<String>,
}
pub fn derive_getter_inner(ast: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let mut getters = vec![];
    if let Data::Struct(DataStruct {
        fields: Fields::Named(FieldsNamed { named: fields, .. }),
        ..
    }) = &ast.data
    {
        for field in fields {
            let field_args = FieldArgs::from_field(field)?;
            if field_args.skip.is_present() {
                continue;
            }
            let field_ident = &field.ident;
            let ty = &field.ty;
            let fn_ident = match field_args.rename {
                Some(rename) => {
                    format_ident!("{}", rename)
                }
                None => {
                    format_ident!("get_{}", field_ident.as_ref().unwrap())
                }
            };
            let (ty, stmt) = if field_args.copy.is_present() {
                (
                    quote! {#ty},
                    quote! {
                        self.#field_ident
                    },
                )
            } else {
                (
                    quote! {&#ty},
                    quote! {
                        &self.#field_ident
                    },
                )
            };

            getters.push(quote! {
                #[inline]
                #[allow(dead_code)]
                pub fn #fn_ident(&self) -> #ty {
                    #stmt
                }
            });
        }
    } else {
        syn_err!(ast, "only surpport named fields struct!");
    }
    Ok(quote! {
        impl #impl_generics #struct_ident #ty_generics #where_clause {
            #(#getters)*
        }
    })
}
#[cfg(test)]
mod test {
    use crate::derive_getter_inner;
    use quote::ToTokens;
    use syn::parse_quote;

    #[test]
    fn test_derive_getter_expand() {
        let code = parse_quote! {
            struct A {
                #[getter(copy)]
                b: i32
            }
        };
        let result = derive_getter_inner(&code).unwrap();
        println!("result is: {}", result.to_token_stream());
    }
}
