// 作为一个真正的 OS 在裸机上运行，为此我们需要移除对标准库的依赖，使其成为一个不依赖当前 OS 功能的库
#![no_std] //指示编译器该程序不使用 Rust 标准库
           //alloc crate 是 Rust 的一个核心库（core library），它提供了一些基本的内存分配器，允许开发者在不使用标准库的情况下进行内存分配。
           //但不同于其他核心库，alloc是独立于核心库的其他部分编译的，所以在no_std情况下，需要显式的引用它。
extern crate alloc; //当使用 #![no_std] 时，由于不链接标准库，一些在标准库中定义的全局分配器和内存分配相关的功能将不可用。此时，alloc crate 可以作为一个替代品，提供基本的内存分配功能。

pub mod object; //包含模块object中的代码
use crate::object::object_imp::DummyObject; //引入空对象路径
use crate::object::KernelObject; //为了方便地使用特性提供的方法，记得也加上特性的路径
use alloc::sync::Arc; //Atomic Reference Counting 引入原子引用计数智能指针。

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    ///创建两个空对象，检查他们的id是否相等，类型名是不是dummyobject,名字是不是default生产的空字符串，然后给o1命名，并检查命名是否成功。
    fn dummy_object() {
        let o1 = DummyObject::new();
        let o2 = DummyObject::new();
        assert_ne!(o1.id(), o2.id());
        assert_eq!(o1.type_name(), "DummyObject");
        assert_eq!(o1.name(), "");
        o1.set_name("object1");
        assert_eq!(o1.name(), "object1");
    }

    // src/object/object.rs
    #[test]
    fn downcast() {
        let dummy = DummyObject::new();
        let object: Arc<dyn KernelObject> = dummy;  //向上转换是简单且i自动的。
        let _result: Arc<DummyObject> = object.downcast_arc::<DummyObject>().unwrap();
    }

}
