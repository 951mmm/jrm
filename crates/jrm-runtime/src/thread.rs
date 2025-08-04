use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
    sync::{Arc, Mutex},
    usize,
};

use derive_builder::Builder;
use jrm_macro::define_instructions;
use jrm_parse::{Constant, ConstantPool};

use crate::{
    Error, Result,
    byte_reader::ByteReader,
    frame::{Frame, LocalVarsLike, OperandStackLike},
    heap::{Heap, ObjectRef, array::ArrayValue, instance::FieldValue},
    method_area::MethodArea,
    slot::Slot,
};

#[derive(Clone)]
pub enum ThreadState {
    Running,
    Blocked,
    Terminated,
}
#[derive(Builder)]
pub struct Thread {
    id: u64,
    /// 线程内部的栈保证一定线程安全
    stack: Rc<RefCell<Vec<Frame>>>,
    state: ThreadState,
    constant_pool: Arc<ConstantPool>,
    // TODO 细化锁
    heap: Arc<Mutex<Heap>>,
    // TODO 细化锁
    method_area: Arc<Mutex<MethodArea>>,
}

// FIXME trait的可见性问题
// pub trait会暴露所有方法
impl OperandStackLike for Thread {
    fn pop<T: From<Slot>>(&mut self) -> Result<T> {
        self.current_frame_mut()?.pop()
    }
    fn push<T: Into<Slot>>(&mut self, operand: T) -> Result<()> {
        self.current_frame_mut()?.push(operand)
    }
}

impl LocalVarsLike for Thread {
    fn get<T: From<Slot>>(&self, index: usize) -> Result<T> {
        self.current_frame()?.get(index)
    }
    fn set<T: Into<Slot>>(&mut self, index: usize, operand: T) -> Result<()> {
        self.current_frame_mut()?.set(index, operand)
    }
}

impl ByteReader for Thread {
    fn read_u1(&self) -> Result<u8> {
        self.current_frame()?.read_u1()
    }
    fn read_u2(&self) -> Result<u16> {
        self.current_frame()?.read_u2()
    }
}

impl Thread {
    // 栈帧操作

    fn current_frame_mut(&self) -> Result<RefMut<Frame>> {
        let stack = self.stack.borrow_mut();
        RefMut::filter_map(stack, |s| s.last_mut()).map_err(|_| Error::empty_stack())
    }
    fn current_frame(&self) -> Result<Ref<Frame>> {
        let stack = self.stack.borrow();
        Ref::filter_map(stack, |s| s.last()).map_err(|_| Error::empty_stack())
    }
    fn run(&mut self) {
        todo!()
    }

    // pc操作

    fn inc_pc(&mut self, val: usize) -> Result<()> {
        self.current_frame_mut()?.pc += val;
        Ok(())
    }
    fn set_pc<F>(&mut self, setter: F) -> Result<()>
    where
        F: FnOnce(&usize) -> usize,
    {
        let pc = setter(&self.current_frame()?.pc);
        self.current_frame_mut()?.pc = pc;
        Ok(())
    }

    // 创建对象的helper
    // 保证调用的方法一定是线程安全

    /// WARN 调用线程安全函数
    fn invoke_construct_method_default(class_name: &str) -> Result<ObjectRef> {
        todo!()
    }
    /// WARN 调用线程安全函数
    fn invoke_construct_method_with_args(
        class_name: &str,
        full_signature: &str,
        args: &[Slot],
    ) -> Result<ObjectRef> {
        todo!()
    }

    // helper

    fn get_slot_from_constant_pool(&mut self, index: u16) -> Result<Slot> {
        if let Some(constant) = self.constant_pool.get(index) {
            let slot = match constant {
                Constant::Integer(integer) => integer.bytes.into(),
                Constant::Float(float) => float.bytes.into(),
                Constant::Long(long) => (long.high_bytes, long.low_bytes).into(),
                Constant::Double(double) => (double.high_bytes, double.low_bytes).into(),
                Constant::String(string) => {
                    let ref_index = string.get_string_index();
                    let utf8_string = self.constant_pool.get_utf8_string(ref_index);
                    self.get_or_create_string_ref(utf8_string)?.into()
                }
                Constant::Class(class) => {
                    let ref_index = class.get_name_index();
                    let class_name = self.constant_pool.get_utf8_string(ref_index);

                    todo!()
                }
                _ => {
                    return Err(Error::InnerError(format!(
                        "invalid constant: {:?}",
                        constant
                    )));
                }
            };
            Ok(slot)
        } else {
            Err(Error::InnerError(format!(
                "constant not found at index: {}",
                index
            )))
        }
    }

    /// 根据lit（字面量）建一一个字符串
    /// 过程
    /// 访问字符串池，是否初始化
    /// 如果需要，根据class创建新的实例
    /// 然后设置value和coder字段
    /// value字段是一个字符数组
    fn get_or_create_string_ref(&mut self, lit: String) -> Result<ObjectRef> {
        if let Some(object_ref) = self.heap.lock()?.get_string_ref(&lit) {
            return Ok(object_ref);
        }

        let (array_value, length, coder) = match helper::get_utf8_string_type(lit) {
            helper::StringType::Latin1(bytes) => {
                let bytes: Vec<_> = bytes.into_iter().map(|byte| byte as i8).collect();
                let length = bytes.len();
                (ArrayValue::Byte(bytes), length, 0)
            }

            helper::StringType::Utf16(utf16) => {
                let length = utf16.len();
                (ArrayValue::Char(utf16), length, 1)
            }
        };

        let coder = FieldValue::Byte(coder);

        let array_ref = self
            .heap
            .lock()?
            .allocate_array_with_value(array_value, length as i32);

        todo!()
    }

    /// 创建字符串对象
    ///
    /// 加载String类
    /// 设置类的字段
    fn create_string_ref(&mut self) -> Result<ObjectRef> {
        const STRING_DESCRIPTOR: &str = "LJava/lang/String";
        todo!()
    }
}

define_instructions! {
    0x00 => nop {
        fn nop() { self.inc_pc(1)?; }
    };
    //0x01
    0x02 => iconst_m1 {
        fn iconst_m1() {
            self.push::<i32>(-1)?;
            self.inc_pc(1)?;
        }
    };
    // ...
    0x18 => ldc {
        fn ldc() {
            self.inc_pc(1)?;
            let index = self.read_u1()? as u16;
            let slot = self.get_slot_from_constant_pool(index)?;
            self.push(slot)?;
            self.inc_pc(1)?;
            // 如果是字符串，就存到string_pool中
        }
    }
}

#[cfg(test)]
impl Thread {
    pub fn set_stack(&mut self, stack: Vec<Frame>) {
        let stack = Rc::new(RefCell::new(stack));
        self.stack = stack;
    }
    pub fn set_code(&mut self, code: Vec<u8>) {
        self.current_frame_mut().unwrap().set_code(code);
    }
}

mod helper {
    pub enum StringType {
        Latin1(Vec<u8>),
        Utf16(Vec<u16>),
    }

    pub fn get_utf8_string_type(utf8_stirng: String) -> StringType {
        if utf8_stirng.chars().all(|c| c as u32 <= 0xFE) {
            StringType::Latin1(utf8_stirng.into_bytes())
        } else {
            let utf16 = utf8_stirng.encode_utf16().collect();
            StringType::Utf16(utf16)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        cell::RefCell,
        path::Path,
        rc::Rc,
        sync::{Arc, Mutex},
    };

    use jrm_parse::{Constant, ConstantClass, ConstantPool, ConstantString};
    use rstest::{fixture, rstest};

    use crate::{
        frame::{FrameBuilder, LocalVars, OperandStack},
        method_area::MethodArea,
        thread::{Thread, ThreadBuilder, ThreadState},
    };

    #[fixture]
    fn thread() -> Thread {
        let constant_pool = ConstantPool::from(vec![
            Constant::Invalid,
            Constant::Class(ConstantClass::new(4)),
            Constant::String(ConstantString::new(3)),
            Constant::from("some string".to_string()),
        ]);
        let frame = FrameBuilder::default()
            .pc(0)
            .operand_stack(OperandStack::new(10))
            .locals(LocalVars::new())
            .code(Arc::new(vec![0x00]))
            .current_class_name(Arc::new("Object".to_string()))
            .build()
            .unwrap();
        let stack = Rc::new(RefCell::new(vec![frame]));
        let constant_pool = Arc::new(constant_pool);

        let method_area = Arc::new(Mutex::new(
            MethodArea::new(Path::new(env!("JAVA_HOME"))).unwrap(),
        ));
        ThreadBuilder::default()
            .id(0)
            .stack(stack)
            .heap(Default::default())
            .state(ThreadState::Running)
            .constant_pool(constant_pool)
            .method_area(method_area)
            .build()
            .unwrap()
    }

    #[rstest]
    fn test_nop(mut thread: Thread) {
        thread.execute_nop().unwrap();
        assert_eq!(thread.current_frame().unwrap().pc, 1);
    }
    #[rstest]
    fn test_iconst_m1(mut thread: Thread) {
        thread.execute_iconst_m1().unwrap();
        assert_eq!(thread.current_frame().unwrap().pc, 1);
        assert_eq!(thread.current_frame().unwrap().top::<i32>(), -1)
    }
    #[rstest]
    fn test_ldc_string(mut thread: Thread) {
        thread.set_code(vec![0x00, 0x02]);
        thread.execute_ldc().unwrap();
    }
}
