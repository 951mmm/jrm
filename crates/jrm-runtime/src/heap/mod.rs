pub mod array;
pub mod instance;

use std::{collections::HashMap, sync::Arc};

use indexmap::IndexMap;

use instance::Instance;

use crate::heap::array::Array;

#[derive(Debug, Default)]
pub struct Heap {
    data: IndexMap<i32, HeapValue>,
    string_pool: HashMap<String, ObjectRef>,
}

impl Heap {
    pub fn get_string_ref(&self, lit: &str) -> Option<ObjectRef> {
        self.string_pool.get(lit).copied()
    }
}

#[derive(Debug)]
pub enum HeapValue {
    Object(Instance),
    Arr(Array),
}
#[derive(Debug)]
pub struct ObjectHeader {
    mark_word: usize,
    // 指向堆中的类
    class_ref: Arc<Instance>,
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
