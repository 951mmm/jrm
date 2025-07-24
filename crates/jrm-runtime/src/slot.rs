use crate::heap::ObjectRef;

#[derive(Debug, Clone)]
pub enum Slot {
    Bits32(u32),
    Bits64(u64),
    Ref(ObjectRef),
}

macro_rules! convert_panic {
    ($ty: ty) => {
        panic!("failed to convert {}", stringify!($ty))
    };
}
impl From<Slot> for u32 {
    fn from(value: Slot) -> Self {
        match value {
            Slot::Bits32(bits) => bits,
            _ => convert_panic!(u32),
        }
    }
}
impl From<Slot> for i32 {
    fn from(value: Slot) -> Self {
        match value {
            Slot::Bits32(bits) => bits as i32,
            _ => convert_panic!(i32),
        }
    }
}
impl From<Slot> for f32 {
    fn from(value: Slot) -> Self {
        match value {
            Slot::Bits32(bits) => f32::from_bits(bits),
            _ => convert_panic!(f32),
        }
    }
}
impl From<Slot> for i64 {
    fn from(value: Slot) -> Self {
        match value {
            Slot::Bits64(bits) => bits as i64,
            _ => convert_panic!(i64),
        }
    }
}

impl From<Slot> for f64 {
    fn from(value: Slot) -> Self {
        match value {
            Slot::Bits64(bits) => f64::from_bits(bits),
            _ => convert_panic!(f64),
        }
    }
}

impl From<u32> for Slot {
    fn from(value: u32) -> Self {
        Slot::Bits32(value)
    }
}

impl From<i32> for Slot {
    fn from(value: i32) -> Self {
        Slot::Bits32(value as u32)
    }
}

impl From<f32> for Slot {
    fn from(value: f32) -> Self {
        Slot::Bits32(value.to_bits())
    }
}

impl From<i64> for Slot {
    fn from(value: i64) -> Self {
        Slot::Bits64(value as u64)
    }
}

impl From<f64> for Slot {
    fn from(value: f64) -> Self {
        Slot::Bits64(value.to_bits())
    }
}

impl From<(u32, u32)> for Slot {
    fn from(value: (u32, u32)) -> Self {
        let (high, low) = value;
        Self::Bits64(((high as u64) >> 32) | (low as u64))
    }
}

impl From<ObjectRef> for Slot {
    fn from(value: ObjectRef) -> Self {
        Self::Ref(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::slot::Slot;

    #[test]
    fn test_slot_into() {
        let slot = Slot::Bits32(0);
        let i: i32 = slot.into();
        assert_eq!(i, 0);
    }
    #[test]
    fn test_slot_from() {
        let i: i32 = 100;
        let slot: Slot = i.into();
        if let Slot::Bits32(bits) = slot {
            if bits == 100 {
                return;
            }
        }
        panic!("error!");
    }
    #[test]
    #[should_panic(expected = "failed to convert f64")]
    fn test_slot_panic() {
        let slot = Slot::Bits32(0);
        let _f: f64 = slot.into();
    }
}
