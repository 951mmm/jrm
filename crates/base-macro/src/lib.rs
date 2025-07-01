use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{LitStr, parse_macro_input};

// struct SimpleFieldAttr {
//     pub field_ident: Ident,
//     pub attr_lit: LitStr,
// }
// impl Parse for SimpleFieldAttr {
//     fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
//         let field_ident: Ident = input.parse()?;
//         input.parse::<Token![,]>()?;
//         let attr_lit: LitStr = input.parse()?;
//         Ok(SimpleFieldAttr {
//             field_ident,
//             attr_lit,
//         })
//     }
// }
#[proc_macro]
pub fn simple_field_attr(input: TokenStream) -> TokenStream {
    let arg = parse_macro_input!(input as LitStr);
    let fn_ident = format_ident!("attr_{}", arg.value());
    quote! {
        fn #fn_ident(field: &syn::Field) -> bool {
            field.attrs.iter().any(|attr| attr.path().is_ident(#arg))
        }
    }
    .into()
}
