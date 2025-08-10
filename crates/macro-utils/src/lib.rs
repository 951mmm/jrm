use prettyplease::unparse;
use syn::{Attribute, File};
pub fn format_tokens(input: proc_macro2::TokenStream) -> String {
    let syn_tree = syn::parse2::<File>(input).unwrap();

    unparse(&syn_tree)
}
pub fn print_expanded_fmt(expanded: proc_macro2::TokenStream) {
    println!("expanded:\n{}", format_tokens(expanded));
}

#[macro_export]
macro_rules! syn_err {
    ($fmt_str: literal $(, $args: expr)*) => {
        return Err(syn::Error::new(proc_macro2::Span::mixed_site(), format_args!($fmt_str $(,$args)*)))
    };
    ($ident: ident, $fmt_str: literal $(, $args: expr)*) => {
        return Err(syn::Error::new_spanned($ident, format_args!($fmt_str, $(,$args)*)))
    };
}

#[macro_export]
macro_rules! unwrap_err {
    ($expr: expr) => {
        match $expr {
            Ok(tokens) => tokens.into(),
            Err(err) => err.to_compile_error().into(),
        }
    };
}

pub trait FromAttrs {
    fn from_attrs(attrs: &[Attribute]) -> syn::Result<Self>
    where
        Self: Sized;
}
