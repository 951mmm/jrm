use prettyplease::unparse;
use syn::File;
pub fn format_tokens(input: proc_macro2::TokenStream) -> String {
    let syn_tree = syn::parse2::<File>(input).unwrap();

    unparse(&syn_tree)
}
pub fn print_expanded_fmt(expanded: proc_macro2::TokenStream) {
    println!("expanded:\n{}", format_tokens(expanded));
}
