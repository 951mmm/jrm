mod attr_enum;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Expr, Ident, MetaList, parse_macro_input};

use crate::attr_enum::attr_enum_inner;

/// build simple attr flag
/// for example:
/// ``` rust
/// #[derive(SomeDerive)]
/// struct Some {
///     #[some_derive(flag)]
///     some: Ty
/// }
/// ```
/// proc macro lib:
/// ``` rust
/// simple_field_attr!{some_derive(flag)}
///
/// fn derive_some_derive(ast: &DataStruct) -> proc_macro2::TokenStream {
///     for field in &ast.fields {
///         let is_flag = flag_from_attrs(&field.attrs);
///         let stmt = if is_flag {
///             generate_with_flag()
///         }
///         else {
///             generate_without_flag()
///         };
///     }
/// }
/// ```
#[proc_macro]
pub fn simple_field_attr(input: TokenStream) -> TokenStream {
    let meta_list = parse_macro_input!(input as MetaList);
    let wrapper_ident = meta_list.path.get_ident();
    let attr_ident: Ident = meta_list.parse_args().unwrap();
    let fn_ident = format_ident!("{}_from_attrs", attr_ident);
    quote! {
        fn #fn_ident(attrs: &[syn::Attribute]) -> bool {
            for attr in attrs {
                if attr.path().is_ident(stringify!(#wrapper_ident)) {
                    return attr
                        .parse_args::<syn::Ident>()
                        .map_or(false, |ident| ident == stringify!(#attr_ident));
                }
            }
            return false;
        }
    }
    .into()
}

#[proc_macro]
pub fn unwrap_darling(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);
    quote! {
        match #expr {
            Ok(tokens) => tokens,
            Err(err) => return err.write_errors().into(),
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn attr_enum(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as attr_enum::Attrs);
    let mut ast = parse_macro_input!(input as DeriveInput);
    attr_enum_inner(&attrs, &mut ast).into()
}
