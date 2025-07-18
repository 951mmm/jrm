use std::{
    sync::{Arc, Mutex},
    usize,
};

use jrm_macro::define_instructions;
use jrm_parse::{Constant, ConstantPool};

use crate::{
    byte_reader::ByteReader,
    frame::{Frame, LocalVarsLike, OperandStackLike},
    heap::{Heap, ObjectRef},
    method::Method,
    slot::Slot,
};

pub enum ThreadState {
    Running,
    Blocked,
    Terminated,
}
pub struct Thread {
    id: u64,
    stack: Vec<Frame>,
    state: ThreadState,
    constant_pool: Arc<ConstantPool>,
    heap: Arc<Mutex<Heap>>,
}

// FIXME trait的可见性问题
impl OperandStackLike for Thread {
    fn pop<T: From<Slot>>(&mut self) -> T {
        self.current_frame_mut().pop()
    }
    fn push<T: Into<Slot>>(&mut self, operand: T) {
        self.current_frame_mut().push(operand);
    }
}

impl LocalVarsLike for Thread {
    fn get<T: From<Slot>>(&self, index: usize) -> T {
        self.current_frame().get(index)
    }
    fn set<T: Into<Slot>>(&mut self, index: usize, operand: T) {
        self.current_frame_mut().set(index, operand);
    }
}

impl ByteReader for Thread {
    fn read_u1(&self) -> u8 {
        self.current_frame().read_u1()
    }
    fn read_u2(&self) -> u16 {
        self.current_frame().read_u2()
    }
}

impl Thread {
    pub fn new(
        id: u64,
        method: Method,
        constant_pool: Arc<ConstantPool>,
        heap: Arc<Mutex<Heap>>,
    ) -> Self {
        let initial_frame = Frame::new(method, 0);
        {
            let constant_pool = constant_pool.clone();
            Self {
                id,
                stack: vec![initial_frame],
                state: ThreadState::Running,
                constant_pool,
                heap,
            }
        }
    }
    pub fn current_frame_mut(&mut self) -> &mut Frame {
        debug_assert!(!self.stack.is_empty(), "none frame");
        let len = self.stack.len();
        unsafe { self.stack.get_unchecked_mut(len - 1) }
    }
    pub fn current_frame(&self) -> &Frame {
        debug_assert!(!self.stack.is_empty(), "none frame");
        let len = self.stack.len();
        unsafe { self.stack.get_unchecked(len - 1) }
    }
    pub fn run(&mut self) {
        while let ThreadState::Running = self.state {
            if self.stack.is_empty() {
                self.state = ThreadState::Terminated;
                break;
            }
            let opcode = self.current_frame();
        }
        todo!()
    }
    fn inc_pc(&mut self, val: u16) {
        self.current_frame_mut().pc += val;
    }
    fn set_pc<F>(&mut self, setter: F)
    where
        F: FnOnce(&u16) -> u16,
    {
        let pc = setter(&self.current_frame().pc);
        self.current_frame_mut().pc = pc;
    }
    pub fn get_slot_from_constant_pool(&mut self, index: u16) -> Slot {
        self.constant_pool
            .get_with(index, |constant| match constant {
                Constant::Integer(integer) => integer.bytes.into(),
                Constant::Float(float) => float.bytes.into(),
                Constant::Long(long) => (long.high_bytes, long.low_bytes).into(),
                Constant::Double(double) => (double.high_bytes, double.low_bytes).into(),
                Constant::String(string) => {
                    let ref_index = string.string_index;
                    let utf8_string = self.constant_pool.get_utf8_string(ref_index);
                    todo!()
                }
                Constant::Class(class) => {
                    let ref_index = class.name_index;
                    let class_name = self.constant_pool.get_utf8_string(ref_index);
                    todo!()
                }
                _ => todo!(),
            })
    }
}

define_instructions! {
    0x00 => nop {
        fn nop() { self.inc_pc(1) }
    };
    //0x01
    0x02 => iconst_m1 {
        fn iconst_m1() {
            self.push::<i32>(-1);
            self.inc_pc(1);
        }
    };
    // ...
    0x18 => ldc {
        fn ldc() {
            let index = self.read_u1() as usize;

            // 如果是字符串，就存到string_pool中
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use jrm_parse::{Constant, ConstantClass, ConstantPool};
    use rstest::{fixture, rstest};

    use crate::{method::Method, thread::Thread};

    #[fixture]
    fn thread() -> Thread {
        let constant_pool = ConstantPool::from(vec![
            Constant::Invalid,
            Constant::Class(ConstantClass::new(2)),
            Constant::from("some string".to_string()),
        ]);
        let constant_pool = Arc::new(constant_pool);

        let method = Method::with_max_stack(100);

        Thread::new(0, method, constant_pool, Default::default())
    }

    #[rstest]
    fn test_constant_pool_get_with(thread: Thread) {
        let class = thread.constant_pool.get_with(1, |class| {
            if let Constant::Class(class) = class {
                let ref_index = class.name_index;
                let class_name = thread.constant_pool.get_utf8_string(ref_index);
                // let class = thread.heap.get_mut().unwrap().allocate()
            }
            todo!()
        });
    }

    #[rstest]
    fn test_thread_frame(thread: Thread) {
        assert_eq!(thread.id, 0);
    }
    #[rstest]
    #[should_panic(expected = "none frame")]
    fn test_thread_frame_panic(mut thread: Thread) {
        let _ = thread.stack.pop();
        let _ = thread.current_frame();
    }
    #[rstest]
    fn test_nop(mut thread: Thread) {
        thread.execute_nop();
        assert_eq!(thread.current_frame().pc, 1);
    }

    #[rstest]
    fn test_iconst_m1(mut thread: Thread) {
        thread.execute_iconst_m1();
        assert_eq!(thread.current_frame().pc, 1);
        assert_eq!(thread.current_frame().top::<i32>(), -1)
    }
}
