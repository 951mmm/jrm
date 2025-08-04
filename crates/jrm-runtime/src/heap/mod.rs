pub mod array;
pub mod instance;

use std::collections::HashMap;

use indexmap::IndexMap;

use instance::Instance;
use jrm_macro::generate_array_arms;

use crate::{
    Error, Result, Type,
    heap::array::{Array, ArrayValue},
};

#[derive(Debug, Default)]
pub struct Heap {
    address: i32,
    data: IndexMap<i32, HeapValue>,
    string_pool: HashMap<String, ObjectRef>,
    reflection_table: HashMap<ObjectRef, ObjectRef>,
}

impl Heap {
    pub fn get_string_ref(&self, lit: &str) -> Option<ObjectRef> {
        self.string_pool.get(lit).copied()
    }
    fn allocate_address(&mut self) -> i32 {
        self.address += 1;
        self.address
    }
    pub fn allocate_instance(&mut self, instance: Instance) -> ObjectRef {
        let address = self.allocate_address();
        let heap_value = HeapValue::Object(instance);
        self.data.insert(address, heap_value).unwrap();
        ObjectRef::from_address(address)
    }

    #[generate_array_arms(Boolean, Byte, Char, Int, Float, Long, Double)]
    pub fn allocate_array(&mut self, item_ty: &Type, dimensions: &[i32]) -> Result<ObjectRef> {
        if dimensions.is_empty() {
            return Err(Error::HeapError(
                "empty dimesions cannot be empty".to_string(),
            ));
        }
        let length = dimensions[0];
        if length < 0 {
            return Err(Error::HeapError("negative array size".to_string()));
        }
        #[inject]
        let array_value = match item_ty {
            Type::Ref { .. } => ArrayValue::Ref(vec![ObjectRef::null(); length as usize]),
            Type::Array(inner_ty) => {
                if dimensions.len() > 1 {
                    let array_ref = (0..length).try_rfold::<_, _, Result<Vec<_>>>(
                        Vec::new(),
                        |mut acc, _| {
                            let array_ref = self.allocate_array(inner_ty, &dimensions[1..])?;
                            acc.push(array_ref);
                            Ok(acc)
                        },
                    )?;
                    ArrayValue::Ref(array_ref)
                } else {
                    return Err(Error::HeapError(
                        "invalid dimension number for nested array".to_string(),
                    ));
                }
            }
            Type::Void => {
                return Err(Error::HeapError("invalid inner type `void`".to_string()));
            }
        };

        let array = Array::new(length, array_value);
        let address = self.allocate_address();
        self.data.insert(address, HeapValue::Arr(array));
        Ok(ObjectRef::from_address(address))
    }
    pub fn allocate_array_with_value(&mut self, array_value: ArrayValue, length: i32) -> ObjectRef {
        let array = Array::new(length, array_value);
        let address = self.allocate_address();
        self.data.insert(address, HeapValue::Arr(array));
        ObjectRef::from_address(address)
    }
}

#[derive(Debug)]
pub enum HeapValue {
    Object(Instance),
    Arr(Array),
}
#[derive(Debug, Default)]
pub struct ObjectHeader {
    mark_word: usize,
    // TODO 类指针
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
    use rstest::{fixture, rstest};

    use crate::heap::{Heap, array::ArrayValue};

    #[fixture]
    fn heap() -> Heap {
        Heap::default()
    }
    #[rstest]
    fn test_heap_allocate(mut heap: Heap) {
        let object_ref = heap.allocate_array_with_value(ArrayValue::Boolean(vec![true, false]), 2);
        assert_eq!(object_ref.as_address(), 1);
    }
}
