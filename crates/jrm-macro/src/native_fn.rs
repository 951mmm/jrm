use darling::FromMeta;
use quote::{format_ident, quote};
use syn::ItemFn;

#[derive(Debug, FromMeta)]
#[darling(derive_syn_parse)]
pub struct Attrs {
    class_path: String,
}

pub fn native_fn_inner(attrs: &Attrs, item_fn: &mut ItemFn) -> proc_macro2::TokenStream {
    let fn_ident = &item_fn.sig.ident;
    let prefix = &attrs.class_path.replace('.', "_");
    item_fn.sig.ident = format_ident!("JAVA_{}_{}", prefix, fn_ident);
    quote! {
        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        #item_fn
    }
}

#[cfg(test)]
mod tests {
    

    use crate::native_fn::Attrs;

    #[test]
    fn test_attrs() {
        let prefix = "prefix = \"aaa\"";
        let attrs: Attrs = syn::parse_str(prefix).unwrap();
    }
}
