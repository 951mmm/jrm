use crate::runtime::slot::Slot;

pub struct OperandStack {
    stack: Vec<Slot>,
    max_size: u16,
}

pub trait OperandStackLike {
    fn push<T: Into<Slot>>(&mut self, operand: T);
    fn pop<T: From<Slot>>(&mut self) -> T;
}

impl OperandStackLike for OperandStack {
    fn push<T: Into<Slot>>(&mut self, operand: T) {
        debug_assert!(self.stack.len() < self.max_size as usize, "stack overflow");
        self.stack.push(operand.into());
    }
    fn pop<T: From<Slot>>(&mut self) -> T {
        debug_assert!(!self.stack.is_empty(), "empty stack");
        unsafe { self.stack.pop().unwrap_unchecked().into() }
    }
}

impl OperandStack {
    pub fn new(max_size: u16) -> Self {
        Self {
            stack: Default::default(),
            max_size,
        }
    }
}

pub trait LocalVarsLike {
    fn get<T: From<Slot>>(&self, index: usize) -> T;
    fn set<T: Into<Slot>>(&mut self, index: usize, operand: T);
}
pub struct LocalVars {
    local_vars: Vec<Slot>,
}
impl LocalVarsLike for LocalVars {
    fn get<T: From<Slot>>(&self, index: usize) -> T {
        todo!()
    }
    fn set<T: Into<Slot>>(&mut self, index: usize, operand: T) {
        todo!()
    }
}

// TODO 异常处理
#[derive(Debug, Default)]
pub struct Method {
    pub name: String,
    pub descriptor: String,
    pub max_locals: u16,
    pub max_stack: u16,
    pub code: Vec<u8>,
    pub is_static: bool,
}

pub struct Frame {
    operand_stack: OperandStack,
    locals: LocalVars,
    return_pc: u16,
    method: Method,
    pub pc: u16,
}

impl OperandStackLike for Frame {
    fn push<T: Into<Slot>>(&mut self, operand: T) {
        self.operand_stack.push(operand);
    }
    fn pop<T: From<Slot>>(&mut self) -> T {
        self.operand_stack.pop()
    }
}

impl LocalVarsLike for Frame {
    fn get<T: From<Slot>>(&self, index: usize) -> T {
        self.locals.get(index)
    }
    fn set<T: Into<Slot>>(&mut self, index: usize, operand: T) {
        self.locals.set(index, operand);
    }
}

impl Frame {
    pub fn new(method: Method, return_pc: u16) -> Self {
        let operand_stack = OperandStack::new(method.max_stack);

        Self {
            operand_stack,
            locals: LocalVars {
                local_vars: Vec::with_capacity(method.max_locals as usize),
            },
            method,
            return_pc,
            pc: 0,
        }
    }
    #[cfg(test)]
    #[allow(unused)]
    pub fn top<T: From<Slot>>(&self) -> T {
        self.operand_stack.stack.last().unwrap().clone().into()
    }
}

#[cfg(test)]
mod tests {
    use crate::runtime::{
        frame::{Frame, LocalVars, LocalVarsLike, OperandStack, OperandStackLike},
        slot::Slot,
    };

    #[test]
    fn test_operand_stack() {
        let mut operand_stack = OperandStack::new(2);
        let a: i32 = 50;
        operand_stack.push(a);
        let b: i32 = operand_stack.pop();
        assert_eq!(b, 50);
    }
    #[test]
    #[should_panic(expected = "stack overflow")]
    fn test_operand_stack_overflow() {
        let mut operand_stack = OperandStack::new(0);
        operand_stack.push(1);
    }
    #[test]
    #[should_panic(expected = "empty stack")]
    fn test_operand_stack_empty() {
        let mut operand_stack = OperandStack::new(1);
        let _: i32 = operand_stack.pop();
    }
    #[test]
    fn test_frame() {
        let frame = Frame::new(Default::default(), 0);
        assert_eq!(frame.locals.local_vars.capacity(), 0);
        assert_eq!(frame.operand_stack.max_size, 0);
        assert_eq!(frame.return_pc, 0);
    }
}
