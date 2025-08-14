mod attribute_enum;
mod base_attribute;
mod build_enum_input;
mod class_parser;
mod constant;
mod constant_enum;
mod define_constants;
mod define_instrucitons;
mod getter;
mod impl_class_parser_for_vec;
mod klass_debug;
mod native_fn;
mod parse_variant;
mod utils;

use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{
    DeriveInput, Expr, ExprMatch, Ident, Item, ItemEnum, ItemFn, ItemStruct, Local, LocalInit,
    Stmt, Token, Type, parse_macro_input, parse_quote,
};

use macro_utils::unwrap_err;

use crate::attribute_enum::attribute_enum_inner;
use crate::base_attribute::base_attrubute_inner;
use crate::class_parser::derive_class_parser_inner;
use crate::constant::constant_inner;
use crate::constant_enum::constant_enum_inner;
use crate::define_constants::define_constants_inner;
use crate::define_instrucitons::define_instructions_inner;
use crate::getter::derive_getter_inner;
use crate::impl_class_parser_for_vec::impl_class_parser_for_vec_inner;
use crate::klass_debug::klass_debug_derive_inner;
use crate::native_fn::native_fn_inner;
use crate::parse_variant::derive_parse_variant_inner;

#[proc_macro]
pub fn generate_ux(_: TokenStream) -> TokenStream {
    let parse_expr: Vec<(Ident, Type)> = vec![
        (parse_quote!(read_one_byte), parse_quote!(u8)),
        (parse_quote!(read_two_bytes), parse_quote!(u16)),
        (parse_quote!(read_four_bytes), parse_quote!(u32)),
    ];
    let parse_stmts = parse_expr.iter().map(|(call, ty)| {
        quote! {
            impl ClassParser for #ty {
                fn parse(ctx: &mut ParserContext) -> anyhow::Result<Self> {
                    Ok(ctx.class_reader.#call().unwrap_or(0))
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
    unwrap_err!(klass_debug_derive_inner(&ast))
}

/// class parser derive
/// - struct field attribute
///   - `enum_entry`
///     see `enum attribute` part
///   - `constant_pool`
///     - `set`
///       enum `Constant` need use constant_pool
///       as index map. use `constant_pool(set)` to
///       init it in `context`.
///     - `read`
///       constant list should preset the index 0
///       position resolve invalid constant index.
///       use `constant_pool(read)` to config `Vec`
///       derive.
///   - `constant_index`
///     - `setend`
///       set constant_pool size in `context` for
///       `check` function
///     - `check`
///       check if is valid constant_pool index
///       ``` rust
///       struct InstanceKlass {
///           #[class_parser(constant_index(setend))]
///           constant_pool_cnt: u16,
///           constant_pool: ConstantPool,
///         
///       }
///       struct ConstantClass {
///           #[class_parser(constant_index(check))]
///           name_index: u16,
///       }
///       ```
///   - `count`
///     use `set` and `get` to set list cnt and
///     read cnt. `get` will impl ClassParser for
///     `Vec` type.
///     ``` rust
///     struct InstanceKlass {
///         #[class_parser(count(set))]
///         constant_pool_cnt: u16,
///         #[class_parser(count(get))]
///         constant_pool: Vec<Constant>,
///     }
///     ```
///     if it's impled for `Vec<Some>` like `Vec<u16>`
///     use `impld`
///     ``` rust
///     struct Some {
///         #[class_parser(count(set))]
///         come_cnt: u8,
///         #[class_parser(count(impled))]
///         somes: Vec<u32>,
///     }
///     ```
///   - `skip`
///     skip recursive class parser stage.
/// - enum attrbute
///   need `#[enum_entry(..)]` for match parse
///   generating
///   - index in the turple struct
///     ``` rust
///     #[derive(ClassParser)]
///     #[class_parser(enum_entry(index(map = constant_index_map[u8])))]
///     struct enum Constant {
///         Class(ConstantClass)
///     }
///     // inner struct
///     #[derive(ClassParser)]
///     struct ConstantClass {
///         #[class_parser(enum_entry(get))]
///         pub tag: u8
///     }
///     ```
///   - index outside the enum
///     ``` rust
///     // outer struct
///     #[derive(ClassParser)]
///     struct ElementValue {
///         #[class_parser(enum_entry(set))]
///         tag: u8,
///         value: Value,
///     }
///     #[derive(ClassParser)]
///     #[class_parser(enum_entry(index(map = element_type_index_map[u8], outer)))]
///     enum Value {
///         ConstValueIndex {
///             ...
///         },
///         ...
///     }
///     ```
#[proc_macro_derive(ClassParser, attributes(class_parser))]
pub fn derive_class_parser(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as Item);
    unwrap_err!(derive_class_parser_inner(&ast))
}

/// - single
///   ``` rust
///   struct Attribute {
///     attribute_name_index: u16,
///     attribute_length: u32,
///     ident: ty
///   }
///   ```
///   if `ty` is collection, then attribute_length
///   is collection's size.
/// - suffix
///   ``` rust
///   struct Attribute {
///     attribute_name_index: u16,
///     attribute_length: u32,
///     ...
///     count_ident: u16,
///     collection_ident: item_ty
///   }
///   ```
///
///   `collection_ident` is auto-gen.
///   or using `rename` option to rename it
/// - none
///   ``` rust
///   struct Attribute {
///     attribute_name_index: u16,
///     attribute_length: u32
///   }
///  ```
///   attribute with none content
#[proc_macro_attribute]
pub fn base_attribute(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as base_attribute::Attrs);
    let mut item_struct = parse_macro_input!(item as ItemStruct);
    unwrap_err!(base_attrubute_inner(&attrs, &mut item_struct))
}

#[proc_macro]
pub fn attribute_enum(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as build_enum_input::Ast);
    attribute_enum_inner(&ast).into()
}

#[proc_macro]
pub fn impl_class_parser_for_vec(input: TokenStream) -> TokenStream {
    let ty = parse_macro_input!(input as Type);
    unwrap_err!(impl_class_parser_for_vec_inner(&ty))
}

#[proc_macro]
pub fn define_constants(input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as define_constants::Ast);
    define_constants_inner(&mut ast.structs).into()
}
#[proc_macro]
pub fn constant_enum(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as build_enum_input::Ast);
    constant_enum_inner(&ast).into()
}
#[proc_macro_attribute]
pub fn constant(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as constant::Attrs);
    let mut item_struct = parse_macro_input!(input as ItemStruct);
    constant_inner(&attr, &mut item_struct).into()
}

#[proc_macro]
pub fn define_instructions(input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as define_instrucitons::Args);
    define_instructions_inner(&mut ast).into()
}

#[proc_macro_attribute]
pub fn native_fn(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attrs as native_fn::Attrs);
    let mut item_fn = parse_macro_input!(input as ItemFn);
    native_fn_inner(&attrs, &mut item_fn).into()
}

#[proc_macro_attribute]
pub fn generate_array_arms(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let tys = Punctuated::<Ident, Token![,]>::parse_terminated
        .parse(attrs)
        .unwrap();
    let mut item_fn = parse_macro_input!(input as ItemFn);
    for stmt in &mut item_fn.block.stmts {
        if let Stmt::Local(Local { init, attrs, .. }) = stmt {
            if attrs.iter().any(|attr| attr.path().is_ident("inject")) {
                attrs.retain(|attr| !attr.path().is_ident("inject"));
                attrs.iter().for_each(|attr| {
                    println!("attr is: {}", attr.to_token_stream());
                    println!("is inject: {}", attr.path().is_ident("inject"));
                });
                if let Some(LocalInit { expr, .. }) = init {
                    if let Expr::Match(ExprMatch { arms, .. }) = &mut **expr {
                        for ty in &tys {
                            arms.push(parse_quote!(
                            Type::#ty => ArrayValue::#ty(std::vec::Vec::with_capacity(length as usize))
                        ));
                        }
                    }
                }
            }
        }
    }
    // let result = quote! {
    //     #item_fn
    // };
    // println!("result is: {}", result);
    // result.into()
    quote! {
        #item_fn
    }
    .into()
}

#[proc_macro_attribute]
pub fn inject(_: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[proc_macro_derive(Getter, attributes(getter))]
pub fn derive_getter(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    unwrap_err!(derive_getter_inner(&ast))
}

#[proc_macro_derive(ParseVariant, attributes(parse_variant))]
pub fn derive_parse_variant(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as ItemEnum);
    unwrap_err!(derive_parse_variant_inner(&ast))
}
