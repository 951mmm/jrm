use convert_case::{Case, Casing};
use quote::quote;
use syn::{Data, DeriveInput, Ident, parse::Parse, parse_quote};

pub struct Attrs {
    wrapper_ident: Ident,
}

impl Parse for Attrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let wrapper_ident: Ident = input.parse()?;
        Ok(Attrs { wrapper_ident })
    }
}
pub fn attr_enum_inner(attrs: &Attrs, ast: &mut DeriveInput) -> proc_macro2::TokenStream {
    let enum_ident = &ast.ident;
    let enum_ident_string = enum_ident
        .to_string()
        .from_case(Case::Camel)
        .to_case(Case::Snake);

    let mut if_items = vec![];
    if let Data::Enum(ref mut data_enum) = ast.data {
        let variants = &data_enum.variants;
        let mut iter_variants = variants.iter();
        if let Some(first) = iter_variants.next() {
            let first_ident = &first.ident;
            let first_lit = first
                .ident
                .to_string()
                .from_case(Case::Camel)
                .to_case(Case::Snake);
            if_items.push(quote! {
                let result = if op == #first_lit {
                    #enum_ident::#first_ident
                }
            });
        }

        for variant in iter_variants {
            let variant_ident = &variant.ident;
            let variant_lit = variant
                .ident
                .to_string()
                .from_case(Case::Camel)
                .to_case(Case::Snake);
            if_items.push(quote! {
                else if op == #variant_lit {
                    #enum_ident::#variant_ident
                }
            });
        }

        if !if_items.is_empty() {
            if_items.push(quote! {
                else {
                    syn_err!("failed to parse attr `{}`", #enum_ident_string);
                };
            });
        }

        data_enum.variants.push(parse_quote!(None));
    }
    ast.attrs.push(parse_quote!(#[derive(Debug)]));

    let Attrs { wrapper_ident } = attrs;

    quote! {
        #ast
        impl FromAttrs for #enum_ident {
            fn from_attrs(attrs: &[syn::Attribute]) -> syn::Result<Self>
                where Self: Sized
            {
                for attr in attrs {
                    if attr.path().is_ident(stringify!(#wrapper_ident)) {
                        let meta_list: syn::MetaList = attr.parse_args()?;
                        if meta_list.path.is_ident(#enum_ident_string) {
                            let op: Ident = meta_list.parse_args()?;
                            #(#if_items)*
                            return Ok(result);
                        }
                    }
                }
                Ok(#enum_ident::None)
            }
        }
    }
}
