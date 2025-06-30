mod class_parser;

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Fields, Ident, Item, Type, parse_macro_input,
    parse_quote,
};

use crate::class_parser::class_file_parse_derive_inner;

#[proc_macro]
pub fn generate_u_parse(_: TokenStream) -> TokenStream {
    let parse_expr: Vec<(Ident, Type)> = vec![
        (parse_quote!(read_one_byte), parse_quote!(u8)),
        (parse_quote!(read_two_bytes), parse_quote!(u16)),
        (parse_quote!(read_four_bytes), parse_quote!(u32)),
    ];
    let parse_stmts = parse_expr.iter().map(|(call, ty)| {
        quote! {
            impl ClassParser for #ty {
                fn parse(class_reader: &mut ClassReader) -> anyhow::Result<Self> {
                    Ok(class_reader.#call().unwrap_or(0))
                }
            }
        }
    });

    quote! {
        #(#parse_stmts)*
    }
    .into()
}

#[proc_macro_derive(KlassDebug, attributes(hex))]
pub fn klass_debug_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    match klass_debug_derive_inner(&ast) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn klass_debug_derive_inner(ast: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
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

#[proc_macro_derive(
    ClassParser,
    attributes(
        not_zero,
        with_lookup,
        impl_sized,
        lookup_outer,
        sized_wrapper,
        constant_pool
    )
)]
pub fn class_file_parse_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as Item);
    match class_file_parse_derive_inner(&ast) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
#[proc_macro_derive(ConstantConstuctor, attributes(ignored))]
pub fn constant_constructor_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    match constant_constructor_derive_inner(&ast) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
fn constant_constructor_derive_inner(ast: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_ident = &ast.ident;
    let mut constructor_fns = vec![];

    if let Data::Enum(DataEnum { variants, .. }) = &ast.data {
        for variant in variants {
            let variant_ident = &variant.ident;
            let constructor_fn_ident = format_ident!(
                "new_{}",
                variant_ident
                    .to_string()
                    .from_case(Case::Camel)
                    .to_case(Case::Snake)
            );
            let mut temp_idents = vec![];
            let mut temp_tys = vec![];

            let is_ignored = variant
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("ignored"));

            if is_ignored {
                continue;
            }

            match &variant.fields {
                Fields::Named(_) => {
                    unreachable!()
                }
                Fields::Unnamed(fields_unnamed) => {
                    for (index, field) in (&fields_unnamed.unnamed).into_iter().enumerate() {
                        let field_ty = &field.ty;
                        let temp_ident = format_ident!("temp_{}", index);
                        temp_idents.push(temp_ident.clone());
                        temp_tys.push(field_ty.clone());
                    }

                    let constructor_fn_args =
                        temp_idents.iter().zip(temp_tys.iter()).map(|(ident, ty)| {
                            quote! {
                                #ident: #ty
                            }
                        });

                    constructor_fns.push(quote! {
                        pub fn #constructor_fn_ident(tag: u8, #(#constructor_fn_args),*) -> ConstantWrapper {
                            ConstantWrapper {
                                tag,
                                constant: #struct_ident::#variant_ident(#(#temp_idents),*)}
                        }
                    });
                }
                Fields::Unit => {
                    constructor_fns.push(quote! {
                        pub fn #constructor_fn_ident(tag: u8) -> ConstantWrapper {
                            ConstantWrapper {
                                tag,
                                constant: #struct_ident::#variant_ident
                            }
                        }
                    });
                }
            }
        }
    }
    Ok(quote! {
        impl ConstantWrapper {
            #(#constructor_fns)*
        }
    })
}
