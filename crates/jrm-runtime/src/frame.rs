use std::sync::Arc;

use derive_builder::Builder;

use crate::{Error, Result, byte_reader::ByteReader, slot::Slot};
#[derive(Clone)]
pub struct OperandStack {
    stack: Vec<Slot>,
    max_size: u16,
}

pub trait OperandStackLike {
    fn push<T: Into<Slot>>(&mut self, operand: T) -> Result<()>;
    fn pop<T: From<Slot>>(&mut self) -> Result<T>;
}

impl OperandStackLike for OperandStack {
    fn push<T: Into<Slot>>(&mut self, operand: T) -> Result<()> {
        if self.stack.len() as u16 >= self.max_size {
            return Err(Error::StackError("stack overflow".to_string()));
        }
        self.stack.push(operand.into());
        Ok(())
    }
    fn pop<T: From<Slot>>(&mut self) -> Result<T> {
        self.stack.pop().ok_or(Error::empty_stack()).map(Into::into)
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
    fn get<T: From<Slot>>(&self, index: usize) -> Result<T>;
    fn set<T: Into<Slot>>(&mut self, index: usize, operand: T) -> Result<()>;
}

#[derive(Clone)]
pub struct LocalVars {
    local_vars: Vec<Slot>,
}
impl LocalVarsLike for LocalVars {
    fn get<T: From<Slot>>(&self, index: usize) -> Result<T> {
        todo!()
    }
    fn set<T: Into<Slot>>(&mut self, index: usize, operand: T) -> Result<()> {
        todo!()
    }
}

/// 栈帧，方法调用时会触发创建，
/// 然后压入线程的栈中
#[derive(Builder)]
pub struct Frame {
    operand_stack: OperandStack,
    locals: LocalVars,
    /// 程序计数器pc
    pub pc: usize,
    /// 异常pc
    #[builder(setter(skip))]
    ex_pc: Option<usize>,
    /// 保存了方法执行的opcode
    /// TODO 使用生命周期切面
    code: Arc<Vec<u8>>,
    current_class_name: Arc<String>,
}

impl OperandStackLike for Frame {
    fn push<T: Into<Slot>>(&mut self, operand: T) -> Result<()> {
        self.operand_stack.push(operand)
    }
    fn pop<T: From<Slot>>(&mut self) -> Result<T> {
        self.operand_stack.pop()
    }
}

impl LocalVarsLike for Frame {
    fn get<T: From<Slot>>(&self, index: usize) -> Result<T> {
        self.locals.get(index)
    }
    fn set<T: Into<Slot>>(&mut self, index: usize, operand: T) -> Result<()> {
        self.locals.set(index, operand)
    }
}

impl ByteReader for Frame {
    fn read_u1(&self) -> Result<u8> {
        self.code
            .get(self.pc)
            .copied()
            .ok_or(Error::StackError("invalid pc".to_string()))
    }
    fn read_u2(&self) -> Result<u16> {
        let high = self.read_u1()? as u16;
        let low = self
            .code
            .get(self.pc + 1)
            .copied()
            .ok_or(Error::StackError("invalid pc".to_string()))? as u16;
        Ok(high << 8 | low)
    }
}

impl Frame {
    #[cfg(test)]
    #[allow(unused)]
    pub fn top<T: From<Slot>>(&self) -> T {
        self.operand_stack.stack.last().unwrap().clone().into()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::{fixture, rstest};

    use crate::{
        byte_reader::ByteReader,
        frame::{Frame, FrameBuilder, LocalVars, LocalVarsLike, OperandStack, OperandStackLike},
    };

    #[test]
    fn test_operand_stack() {
        let mut operand_stack = OperandStack::new(2);
        let a: i32 = 50;
        operand_stack.push(a).unwrap();
        let b: i32 = operand_stack.pop().unwrap();
        assert_eq!(b, 50);
    }
    #[test]
    #[should_panic(expected = "stack overflow")]
    fn test_operand_stack_overflow() {
        let mut operand_stack = OperandStack::new(0);
        operand_stack.push(1).unwrap();
    }
    #[fixture]
    fn frame() -> Frame {
        FrameBuilder::create_empty()
            .code(Arc::new(vec![0x01, 0x11]))
            .pc(0)
            .operand_stack(OperandStack::new(1))
            .locals(LocalVars { local_vars: vec![] })
            .current_class_name(Arc::new("some".to_string()))
            .build()
            .unwrap()
    }
    #[rstest]
    fn test_frame(frame: Frame) {
        assert_eq!(frame.locals.local_vars.capacity(), 0);
        assert_eq!(frame.operand_stack.max_size, 1);
    }
    #[rstest]
    fn test_frame_read_byte(frame: Frame) {
        let u1 = frame.read_u1().unwrap();
        assert_eq!(u1, 0x01);
        let u2 = frame.read_u2().unwrap();
        assert_eq!(u2, 0x111);
    }
}
