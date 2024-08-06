// 作为一个真正的 OS 在裸机上运行，为此我们需要移除对标准库的依赖，使其成为一个不依赖当前 OS 功能的库
#![no_std] //指示编译器该程序不使用 Rust 标准库
           //alloc crate 是 Rust 的一个核心库（core library），它提供了一些基本的内存分配器，允许开发者在不使用标准库的情况下进行内存分配。
           //但不同于其他核心库，alloc是独立于核心库的其他部分编译的，所以在no_std情况下，需要显式的引用它。
extern crate alloc; //当使用 #![no_std] 时，由于不链接标准库，一些在标准库中定义的全局分配器和内存分配相关的功能将不可用。此时，alloc crate 可以作为一个替代品，提供基本的内存分配功能。

pub mod object; //包含模块object中的代码
use crate::object::object_imp::DummyObject;
use crate::object::KernelObject;
use alloc::sync::Arc;
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn impl_kobject() {
        use alloc::format;
        let dummy = DummyObject::new();
        let object: Arc<dyn KernelObject> = dummy;
        assert_eq!(object.type_name(), "DummyObject");
        assert_eq!(object.name(), "");
        object.set_name("dummy");
        assert_eq!(object.name(), "dummy");
        assert_eq!(
            format!("{:?}", object),
            format!("DummyObject({}, \"dummy\")", object.id())
        );
        let _result: Arc<DummyObject> = object.downcast_arc::<DummyObject>().unwrap();
}

}
