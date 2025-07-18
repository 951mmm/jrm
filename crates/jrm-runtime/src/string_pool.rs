use std::collections::HashMap;

use crate::heap::ObjectRef;

#[derive(Debug, Default)]
pub struct StringPool {
    string_map: HashMap<String, ObjectRef>,
}

impl StringPool {
    pub fn get_string(&self, string: &str) -> Option<ObjectRef> {
        self.string_map.get(string).cloned()
    }
    pub fn put_string(&mut self, string: String, string_ref: ObjectRef) -> Option<ObjectRef> {
        self.string_map.insert(string, string_ref)
    }
}
