use std::{collections::HashMap, sync::Arc};

use derive_builder::Builder;
use jrm_macro::Getter;
use jrm_parse::instance_klass::MethodAccessFlags;

use crate::{Error, Parser, Type, heap::ObjectRef, parser_err};

// TODO 异常处理
#[derive(Getter, Builder)]
pub struct Method {
    class_name: Arc<String>,
    access_flags: MethodAccessFlags,
    #[getter(copy, rename = "is_native")]
    is_native: bool,
    name: String,
    signature: Arc<MethodSignature>,

    #[getter(copy)]
    max_locals: u16,
    #[getter(copy)]
    max_stack: u16,
    code: Arc<Vec<u8>>,
    line_numbers: Arc<HashMap<u16, u16>>,
    exception_table: Vec<Exception>,
    exception_indices: Vec<String>,

    refection_ref: ObjectRef,
}

#[derive(Debug, Clone)]
pub struct Exception {
    start_pc: u16,
    end_pc: u16,
    handler_pc: u16,
    catch_type: CatchType,
}

// 四个参数及一下使用new
impl Exception {
    pub fn new(start_pc: u16, end_pc: u16, handler_pc: u16, catch_type: CatchType) -> Self {
        Self {
            start_pc,
            end_pc,
            handler_pc,
            catch_type,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CatchType {
    Throwable,
    Exception(String),
}

/// 函数签名
/// 暂时支持向前看一位
/// TODO parameters的预查。向前看n位
/// 找到括号，然后过滤[Type::Void]
pub struct MethodSignature {
    parameter_types: Vec<Type>,
    return_type: Type,
}

enum ParserState {
    Start,
    Params,
    Return,
    End,
}

pub trait ContainVoid {
    fn contain_void(&self) -> bool;
}

impl ContainVoid for Type {
    fn contain_void(&self) -> bool {
        match self {
            Type::Void => true,
            Type::Array(inner_ty) => inner_ty.contain_void(),
            _ => false,
        }
    }
}

impl Parser for MethodSignature {
    const NAME: &'static str = "MethodSignature";
    fn parse(iter: &mut std::iter::Peekable<impl Iterator<Item = char>>) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let mut state = ParserState::Start;
        let mut parameter_types = vec![];
        let mut return_type = None;
        loop {
            match state {
                ParserState::Start => match iter.next() {
                    Some('(') => state = ParserState::Params,
                    Some(ch) => {
                        parser_err!("expected `(`, found {}", ch);
                    }
                    None => {
                        parser_err!("unexpected end at start");
                    }
                },
                ParserState::Params => match iter.peek() {
                    Some(')') => {
                        iter.next();
                        state = ParserState::Return;
                    }
                    Some(_) => {
                        let ty = Type::parse(iter)?;
                        if ty.contain_void() {
                            parser_err!("unexpected parameter type: Void");
                        }
                        parameter_types.push(ty);
                    }
                    None => {
                        parser_err!("unexpected end in params");
                    }
                },
                ParserState::Return => {
                    return_type = match iter.peek() {
                        Some(_) => Some(Type::parse(iter)?),
                        None => {
                            parser_err!("no return type")
                        }
                    };
                    state = ParserState::End;
                }
                ParserState::End => {
                    return Ok(Self {
                        parameter_types,
                        return_type: return_type.take().unwrap(),
                    });
                }
            }
        }
    }
}

impl TryFrom<String> for MethodSignature {
    type Error = Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut iter = value.chars().peekable();
        let result = MethodSignature::parse(&mut iter)?;
        if iter.next().is_some() {
            return Err(Error::InnerError(format!(
                "invalid method signature: {}",
                value
            )));
        }
        Ok(result)
    }
}
#[cfg(test)]
mod test {
    use crate::Type;
    use crate::Type::*;
    use crate::util::test_util::{array_type, object_type};
    use crate::{Parser, method::MethodSignature};
    use rstest::rstest;

    // valid test cases
    #[rstest]
    #[case("()V", vec![], Void, "empty params")]
    #[case("(I)V", vec![Int], Void, "single primitive param")]
    #[case("(J)D", vec![Long], Double, "primitive param and return")]
    #[case("(Ljava/lang/String;)V", vec![object_type("java.lang.String")], Void, "object param")]
    #[case("([I)V", vec![array_type(Int)], Void, "primitive array")]
    #[case("([[Ljava/lang/Object;)I", vec![array_type(array_type(object_type("java.lang.Object")))], Int, "multi-dimensional object array")]
    #[case("(FI[Z)J", vec![Float, Int, array_type(Boolean)], Long, "mixed params")]
    #[case("(Lcom/example/MyClass;)Lcom/example/ReturnType;", vec![object_type("com.example.MyClass")], object_type("com.example.ReturnType"), "full object path")]
    fn test_valid_signatures(
        #[case] descriptor: &str,
        #[case] expected_params: Vec<Type>,
        #[case] expected_return: Type,
        #[case] desc: &str,
    ) {
        let mut iter = descriptor.chars().peekable();
        let sig = MethodSignature::parse(&mut iter)
            .unwrap_or_else(|_| panic!("failed to parse: {}", desc));

        assert_eq!(
            sig.parameter_types, expected_params,
            "param types mismatch: {}",
            desc
        );
        assert_eq!(
            sig.return_type, expected_return,
            "return type mismatch: {}",
            desc
        );
        assert!(iter.next().is_none(), "not all chars consumed: {}", desc);
    }

    // error test cases
    #[rstest]
    #[case("", "unexpected end at start", "empty input")]
    #[case("V", "expected `(`, found V", "missing parentheses")]
    #[case("(V)", "unexpected parameter type: Void", "void not allowed in params")]
    #[case("(I)", "no return type", "params not closed")]
    #[case(
        "(LMissingSemicolon)",
        "invalid character: )",
        "missing semicolon in object type"
    )]
    #[case("(I)(", "invalid basic type descriptor: (", "invalid return type")]
    #[case("(X)V", "invalid basic type descriptor: X", "unknown type")]
    #[case("()", "no return type", "missing return type")]
    #[case("(I)Jextra", "invalid method signature", "extra chars")]
    #[case("(II)Iextra", "invalid method signature: ", "extra chars after params")]
    #[case(
        "(Ljava/lang/String)V",
        "invalid character: )",
        "missing semicolon in object type"
    )]
    fn test_invalid_signatures(
        #[case] descriptor: &str,
        #[case] error_msg: &str,
        #[case] desc: &str,
    ) {
        match MethodSignature::try_from(descriptor.to_string()) {
            Ok(_) => panic!("parse should fail: {}", desc),
            Err(e) => assert!(
                e.to_string().contains(error_msg),
                "expected error containing '{}', got '{}' ({})",
                error_msg,
                e,
                desc
            ),
        }
    }
}
