pub mod instance;

use std::{
    collections::HashMap,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};

use indexmap::IndexMap;

use crate::{slot::Slot, string_pool::StringPool};
use instance::Instance;

#[derive(Debug, Default)]
pub struct Heap {
    data: IndexMap<ObjectRef, Instance>,
    string_pool: HashMap<String, ObjectRef>,
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ObjectRef(i32);
impl ObjectRef {
    pub const fn from_address(address: i32) -> Self {
        Self(address)
    }
    pub const fn as_address(self) -> i32 {
        self.0
    }
    pub const fn null() -> Self {
        Self(0)
    }
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }
}

#[cfg(test)]
mod tests {
    use crate::heap::ObjectRef;
}
