mod byte_reader;
mod class_loader;
mod frame;
mod heap;
mod slot;
mod thread;

use std::sync::Arc;

pub use frame::Method;
pub use slot::Slot;

use crate::runtime::{class_loader::ClassLoader, heap::Heap};

pub struct Runtime {
    pub heap: Heap,
    pub bootstrap_loader: Arc<ClassLoader>,
    pub system_loader: Arc<ClassLoader>,
}
