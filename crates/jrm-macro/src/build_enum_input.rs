use syn::{Ident, Token, parse::Parse, punctuated::Punctuated};

pub struct Ast {
    pub idents: Punctuated<Ident, Token![,]>,
}
impl Parse for Ast {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let idents = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;
        Ok(Ast { idents })
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use syn::parse_str;

    use crate::build_enum_input;

    #[test]
    fn test_ast_parse() -> Result<(), Box<dyn Error>> {
        let code = r#"
            A, B, C
        "#;
        let ast: build_enum_input::Ast = parse_str(code)?;
        assert_eq!(ast.idents.len(), 3);
        assert_eq!(ast.idents[1], "B");
        Ok(())
    }
}
