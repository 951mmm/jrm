use std::{mem, sync::PoisonError};

mod byte_reader;
mod class;
mod class_loader;
mod frame;
mod heap;
mod method;
mod method_area;
mod slot;
mod thread;

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
}

impl<T> From<PoisonError<T>> for Error {
    fn from(error: PoisonError<T>) -> Self {
        Self::InnerError(format!("poison error: {error}"))
    }
}

impl Error {
    pub fn empty_stack() -> Self {
        Self::StackError("empty stack".to_string())
    }
}
pub type Result<T> = std::result::Result<T, Error>;

pub enum Type {
    Boolean,
    Byte,
    Char,
    Int,
    Float,
    Long,
    Double,
    Ref,
    Array(Box<Type>),
}

#[derive(Debug)]
pub struct Runtime {}
