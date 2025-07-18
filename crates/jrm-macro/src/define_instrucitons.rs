use quote::{ToTokens, format_ident, quote};
use syn::{
    Ident, ItemFn, Lit, Macro, Token, braced,
    parse::Parse,
    parse_quote,
    punctuated::Punctuated,
};

pub struct Args {
    args: Punctuated<Arg, Token![;]>,
}

impl Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let args = Punctuated::<Arg, Token![;]>::parse_terminated(input)?;
        Ok(Self { args })
    }
}
pub struct Arg {
    opcode: Lit,
    instr_ident: Ident,
    executor: Executor,
}

impl Parse for Arg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let opcode: Lit = input.parse()?;
        input.parse::<Token![=>]>()?;
        let instr_ident: Ident = input.parse()?;
        let content;
        braced!(content in input);
        let executor: Executor = content.parse()?;
        Ok(Self {
            opcode,
            instr_ident,
            executor,
        })
    }
}
pub enum Executor {
    Fn(ItemFn),
    Macro(Macro),
}

impl Parse for Executor {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let executor = if input.peek2(Token![!]) {
            let __macro: Macro = input.parse()?;
            Executor::Macro(__macro)
        } else {
            let __fn: ItemFn = input.parse()?;
            Executor::Fn(__fn)
        };
        Ok(executor)
    }
}
pub fn define_instructions_inner(ast: &mut Args) -> proc_macro2::TokenStream {
    let mut executor_arms = vec![];
    let mut executor_fns = vec![];
    for arg in ast.args.iter_mut() {
        let Arg {
            opcode,
            instr_ident,
            executor,
        } = arg;
        let instr_ident_string = instr_ident.to_string();
        let executor_fn_ident = format_ident!("execute_{}", instr_ident_string);
        let executor_arm = quote! {
            #opcode => self.#executor_fn_ident(),
        };
        executor_arms.push(executor_arm);
        let executor_fn = match executor {
            Executor::Fn(__fn) => {
                __fn.sig.ident = executor_fn_ident;
                __fn.sig.inputs = parse_quote!(&mut self);
                __fn.to_token_stream()
            }
            Executor::Macro(__macro) => __macro.to_token_stream(),
        };
        executor_fns.push(executor_fn);
    }

    quote! {
        impl Thread {
            pub fn execute(&mut self, opcode: u8) {
                match opcode {
                    #(#executor_arms)*
                    _ => unsafe { std::hint::unreachable_unchecked() }
                };
            }
            #(#executor_fns)*
        }
    }
}
