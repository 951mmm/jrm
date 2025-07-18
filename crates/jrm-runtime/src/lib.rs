mod byte_reader;
mod class;
mod class_loader;
mod frame;
mod heap;
mod method;
mod method_area;
mod slot;
mod string_pool;
mod thread;

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("heap error: {0}")]
    HeapError(String),
}

#[derive(Debug)]
pub struct Runtime {}
