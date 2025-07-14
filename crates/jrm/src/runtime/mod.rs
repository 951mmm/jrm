mod byte_reader;
mod frame;
mod heap;
mod slot;
mod thread;

pub use frame::Method;
pub use slot::Slot;

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("illegal state")]
    IllegalState,
}
