
use super::{KernelObject, Rights};
use alloc::sync::Arc;

///句柄是允许用户程序引用内核对象引用的一种内核结构，它可以被认为是与特定内核对象的会话或连接。
///通常情况下，多个进程通过不同的句柄同时访问同一个对象。
/// 对象可能有多个句柄（在一个或多个进程中）引用它们。但单个句柄只能绑定到单个进程或绑定到内核。
#[derive(Clone)]
pub struct Handle {
    pub object: Arc<dyn KernelObject>,
    pub rights: Rights,
}

impl Handle {
    /// 创建一个新句柄
    pub fn new(object: Arc<dyn KernelObject>, rights: Rights) -> Self {
        Handle { object, rights }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::object_imp::DummyObject;
    
    #[test]
    fn new_obj_handle() {
        let obj = DummyObject::new();
        let _handle1 = Handle::new(obj.clone(), Rights::BASIC);
    }
}