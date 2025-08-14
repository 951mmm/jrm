use std::{iter::Peekable, sync::PoisonError};

use crate::{class::ClassBuilderError, method::MethodBuilderError};

mod byte_reader;
mod class;
mod frame;
mod heap;
mod method;
mod method_area;
mod slot;
mod thread;
mod util;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("heap error: {0}")]
    HeapError(String),
    #[error("stack error: {0}")]
    StackError(String),
    #[error("inner error: {0}")]
    InnerError(String),
    #[error("execution error: {0}")]
    ExecutionError(String),
    #[error("class loader error: {0}")]
    ClassLoaderError(#[from] anyhow::Error),
}

impl<T> From<PoisonError<T>> for Error {
    fn from(error: PoisonError<T>) -> Self {
        Self::InnerError(format!("poison error: {error}"))
    }
}

impl From<ClassBuilderError> for Error {
    fn from(value: ClassBuilderError) -> Self {
        Self::ClassLoaderError(value.into())
    }
}

impl From<MethodBuilderError> for Error {
    fn from(value: MethodBuilderError) -> Self {
        Self::ClassLoaderError(value.into())
    }
}

impl Error {
    pub fn empty_stack() -> Self {
        Self::StackError("empty stack".to_string())
    }
    pub fn parse_descriptor(e: String) -> Self {
        Self::InnerError(format!("parse descriptor error: {}", e))
    }
}
pub type Result<T> = std::result::Result<T, Error>;

/// Java类型
/// 能和类型描述符相互转换
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Boolean,
    Byte,
    Char,
    Int,
    Float,
    Long,
    Double,
    Ref {
        binary_name: String,
        descriptor: String,
    },
    Array(Box<Type>),
    Void,
}

impl PartialEq<&str> for Type {
    fn eq(&self, other: &&str) -> bool {
        String::from(self.clone()) == *other.to_string()
    }
}

pub trait Parser {
    const NAME: &'static str;
    fn parse(iter: &mut Peekable<impl Iterator<Item = char>>) -> Result<Self>
    where
        Self: Sized;
}

#[derive(PartialEq, Eq)]
enum ParserState {
    Start,
    Array,
    Object,
    Basic,
    ArrayCheck,
    End,
}

const MAX_ARRAY_DIMENSIONS: usize = 255;

impl Parser for Type {
    const NAME: &'static str = "Type";
    fn parse(iter: &mut Peekable<impl Iterator<Item = char>>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut array_stack_top = 0;
        let mut type_stack = None;

        let mut state = ParserState::Start;

        loop {
            match state {
                ParserState::Start => {
                    let ch = iter.peek().unwrap();
                    if ch == &'[' {
                        state = ParserState::Array;
                        iter.next();
                    } else if ch == &'L' {
                        state = ParserState::Object;
                    } else {
                        state = ParserState::Basic;
                    }
                }
                ParserState::Array => {
                    array_stack_top += 1;
                    if array_stack_top > MAX_ARRAY_DIMENSIONS {
                        parser_err!(
                            "invalid array dismensions, exceeded limit of {}",
                            MAX_ARRAY_DIMENSIONS
                        );
                    }
                    state = ParserState::Start;
                }
                ParserState::Object => {
                    let helper::ClassName {
                        binary_name,
                        descriptor,
                    } = helper::ClassName::parse(iter)?;
                    type_stack = Some(Type::Ref {
                        binary_name,
                        descriptor,
                    });
                    state = ParserState::ArrayCheck;
                }
                ParserState::Basic => {
                    let ch = iter.next();
                    let ty = match ch {
                        Some('Z') => Type::Boolean,
                        Some('B') => Type::Byte,
                        Some('C') => Type::Char,
                        Some('I') => Type::Int,
                        Some('F') => Type::Float,
                        Some('J') => Type::Long,
                        Some('D') => Type::Double,
                        Some('V') => Type::Void,
                        None => {
                            parser_err!("undefined error!");
                        }
                        _ => {
                            parser_err!("invalid basic type descriptor: {}", ch.unwrap());
                        }
                    };
                    type_stack = Some(ty);
                    state = ParserState::ArrayCheck;
                }
                ParserState::ArrayCheck => {
                    if type_stack.is_none() {
                        parser_err!("undefined erorr!");
                    }
                    let mut ty = type_stack.take().unwrap();
                    while array_stack_top > 0 {
                        array_stack_top -= 1;
                        ty = Type::Array(Box::new(ty));
                    }
                    type_stack = Some(ty);
                    state = ParserState::End;
                }
                ParserState::End => return Ok(type_stack.unwrap()),
            }
        }
    }
}

impl TryFrom<String> for Type {
    type Error = Error;
    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        let mut iter = value.chars().peekable();
        let result = Type::parse(&mut iter)?;
        if iter.next().is_some() {
            return Err(Error::InnerError(format!("invalid descriptor: {}", value)));
        }
        Ok(result)
    }
}

enum TypeEmitState {
    Start,
    End,
}

impl From<Type> for String {
    fn from(mut value: Type) -> Self {
        let mut chars = vec![];
        let mut state = TypeEmitState::Start;
        loop {
            match state {
                TypeEmitState::Start => {
                    if let Type::Array(inner_ty) = value {
                        value = *inner_ty;
                        chars.push('[');
                        continue;
                    } else if let Type::Ref { descriptor, .. } = &value {
                        chars.extend(descriptor.chars());
                    } else {
                        let ch = match value {
                            Type::Boolean => 'Z',
                            Type::Byte => 'B',
                            Type::Char => 'C',
                            Type::Int => 'I',
                            Type::Float => 'F',
                            Type::Long => 'J',
                            Type::Double => 'D',
                            Type::Void => 'V',
                            _ => unreachable!(),
                        };
                        chars.push(ch);
                    }
                    state = TypeEmitState::End;
                    continue;
                }
                TypeEmitState::End => {
                    return chars.iter().collect();
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Runtime {}

mod helper {
    use jrm_parse::{Constant, ConstantPool};

    use crate::{Error, Parser, Result, parser_err};

    pub trait TryGetConstant {
        fn try_get(&self, index: u16) -> Result<&Constant>;
    }

    impl TryGetConstant for ConstantPool {
        fn try_get(&self, index: u16) -> Result<&Constant> {
            self.get(index).ok_or(Error::InnerError(format!(
                "failed to get constant at index: {}",
                index
            )))
        }
    }

    pub struct ClassName {
        pub binary_name: String,
        pub descriptor: String,
    }

    #[derive(Debug)]
    enum ParserState {
        Start,
        AfterL,      // 已读取 'L'
        AfterSlash,  // 已读取 '/'
        InName,      // 在类名部分
        AfterDollar, // 刚读取 '$'
        End,         // 成功解析完成
    }
    impl Parser for ClassName {
        const NAME: &'static str = "Type::Ref{..}";
        fn parse(iter: &mut std::iter::Peekable<impl Iterator<Item = char>>) -> Result<Self>
        where
            Self: Sized,
        {
            let mut state = ParserState::Start;
            let mut class_name = String::new();
            loop {
                match state {
                    ParserState::Start => {
                        match iter.peek() {
                            Some('L') => {
                                iter.next(); // 消耗 'L'
                                state = ParserState::AfterL;
                            }
                            _ => {
                                parser_err!("descriptor must start with `L`");
                            }
                        }
                    }

                    ParserState::AfterL => match iter.next() {
                        Some(ch) if valid_name_char(ch) => {
                            class_name.push(ch);
                            state = ParserState::InName;
                        }
                        Some(ch) => {
                            parser_err!("invalid class name start: {}", ch);
                        }
                        None => {
                            parser_err!("unexpected end after `L`");
                        }
                    },

                    ParserState::InName => match iter.peek() {
                        Some(';') => {
                            iter.next();
                            state = ParserState::End;
                        }
                        Some('/') => {
                            class_name.push(iter.next().unwrap());
                            state = ParserState::AfterSlash;
                        }
                        Some('$') => {
                            iter.next();
                            class_name.push('$');
                            state = ParserState::AfterDollar;
                        }
                        Some(ch) if valid_name_char(*ch) => {
                            class_name.push(iter.next().unwrap());
                        }
                        Some(ch) => {
                            parser_err!("invalid character: {}", ch);
                        }
                        None => {
                            parser_err!("unexpected end");
                        }
                    },

                    ParserState::AfterSlash => match iter.peek() {
                        Some(ch) if valid_name_char(*ch) => {
                            state = ParserState::InName;
                        }
                        Some(ch) => {
                            parser_err!("invalid character after `/`: {}", ch);
                        }
                        None => {
                            parser_err!("unexpected end after `/`");
                        }
                    },

                    ParserState::AfterDollar => match iter.peek() {
                        Some(';') => {
                            parser_err!("cannot end with `$`");
                        }
                        Some(ch) if valid_name_char(*ch) => {
                            class_name.push(iter.next().unwrap());
                            state = ParserState::InName;
                        }
                        Some(ch) => {
                            parser_err!("invalid character after `$`: {}", ch);
                        }
                        None => {
                            parser_err!("unexpected end after `$`");
                        }
                    },

                    ParserState::End => {
                        let binary_name = class_name.replace('/', ".");
                        let descriptor = format!("L{};", class_name);
                        return Ok(Self {
                            binary_name,
                            descriptor,
                        });
                    }
                }
            }
        }
    }
    /// 检查是否是部分合法的类名字符
    fn valid_name_char(ch: char) -> bool {
        ch.is_ascii_alphanumeric() || ch == '_'
    }
}

#[cfg(test)]
mod test {
    use crate::Type;
    use crate::Type::*;
    use crate::util::test_util::{array_type, object_type};
    use rstest::rstest;

    #[rstest]
    #[case("[[I", array_type(array_type(Int)), "nested array")]
    #[case(
        "[Llang/some/demo;",
        array_type(object_type("lang.some.demo")),
        "object array"
    )]
    #[case(
        "Llang/some/demo$inner;",
        object_type("lang.some.demo$inner"),
        "inner class"
    )]
    #[case("Z", Boolean, "renamed basic type")]
    fn test_type_try_from(#[case] input: &str, #[case] expected: Type, #[case] desc: &str) {
        let ty = Type::try_from(input.to_string()).unwrap();
        assert_eq!(ty, expected, "{desc}");
    }

    #[rstest]
    #[case("M", "basic")] // 非法基础类型
    #[case("[".repeat(256) + "I", "exceeded dimensions")] // 数组维度超限
    #[case("Ljava/lang/ObjectEXTRA", "missing `;`")] // 缺少分号
    #[case("Ljava/lang/Object;;", "bad terminated end")] // 多余分号
    #[case("Llang/some/$demo", "bad class name begin with `$`")]
    #[case("Llang/some/demo$;", "unexpected end")]
    fn test_invalid_descriptor(#[case] input: String, #[case] desc: &str) {
        assert!(Type::try_from(input).is_err(), "{desc}")
    }

    #[rstest]
    #[case(array_type(array_type(Int)), "[[I")]
    #[case(array_type(object_type("lang.some.demo")), "[Llang/some/demo;")]
    #[case(Type::Boolean, "Z")]
    fn test_type_into(#[case] input: Type, #[case] expect: &str) {
        let descriptor: String = input.into();
        assert_eq!(descriptor, expect);
    }
}
