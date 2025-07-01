use convert_case::{Case, Casing};
use quote::quote;
use syn::{Data, DataStruct, DeriveInput};

pub fn klass_debug_derive_inner(ast: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let klass_ident = &ast.ident;
    let mut field_stmts = vec![];

    if let Data::Struct(DataStruct { fields, .. }) = &ast.data {
        for field in fields {
            let field_ident = &field.ident;
            let field_ident_lit = field_ident.clone().unwrap().to_string();
            let title_case_lit = field_ident_lit.from_case(Case::Snake).to_case(Case::Title);

            let is_hex = field.attrs.iter().any(|attr| attr.path().is_ident("hex"));

            if is_hex {
                field_stmts.push(quote! {
                    writeln!(f, "  {}: 0X{:X}", #title_case_lit, self.#field_ident)?;
                });
            } else {
                field_stmts.push(quote! {
                    writeln!(f, "  {}: {:?}", #title_case_lit, self.#field_ident)?;
                });
            }
        }
    }

    let result = quote! {
        impl std::fmt::Debug for #klass_ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                writeln!(f, "InstantKlass:\n")?;
                #(#field_stmts;)*
                Ok(())
            }
        }
    };

    Ok(result)
}
