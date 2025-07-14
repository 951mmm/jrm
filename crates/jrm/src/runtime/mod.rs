mod frame;
mod slot;
mod thread;

pub use frame::Method;

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("illegal state")]
    IllegalState,
}
