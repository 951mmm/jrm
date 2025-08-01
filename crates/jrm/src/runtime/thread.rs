use std::{ops::Index, sync::Arc, usize};

use jrm_macro::define_instructions;

use crate::{
    constant_pool::ConstantPool,
    runtime::{
        Method,
        frame::{Frame, LocalVarsLike, OperandStackLike},
        slot::Slot,
    },
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

impl Thread {
    pub fn new(id: u64, method: Method, constant_pool: Arc<ConstantPool>) -> Self {
        let initial_frame = Frame::new(method, 0);
        {
            let constant_pool = constant_pool.clone();
            Self {
                id,
                stack: vec![initial_frame],
                state: ThreadState::Running,
                constant_pool: constant_pool,
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

        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::{fixture, rstest};

    use crate::{
        constant_pool::{Constant, ConstantPool},
        runtime::{Method, thread::Thread},
    };

    #[fixture]
    fn thread() -> Thread {
        let constant_pool = ConstantPool::from(vec![
            Constant::Invalid,
            Constant::from("some string".to_string()),
        ]);
        let constant_pool = Arc::new(constant_pool);

        let method = Method {
            max_stack: 100,
            ..Default::default()
        };
        let thread = Thread::new(0, method, constant_pool);
        thread
    }

    #[rstest]
    fn test_thread_frame(thread: Thread) {
        assert_eq!(thread.id, 11);
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
