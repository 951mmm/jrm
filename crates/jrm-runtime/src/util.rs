#[macro_export]
macro_rules! parser_err {
    ($($args: expr),+) => {
        return Err(Error::InnerError(
        format!("occur error when parse {}, {}", Self::NAME, format_args!($($args),+))
        ))
    }
}

#[cfg(test)]
pub mod test_util {
    use crate::Type;
    pub fn object_type(name: &str) -> Type {
        let binary_name = name.to_string();
        let descriptor = format!("L{};", name.replace('.', "/"));
        Type::Ref {
            binary_name,
            descriptor,
        }
    }
    pub fn array_type(inner: Type) -> Type {
        Type::Array(Box::new(inner))
    }
}
