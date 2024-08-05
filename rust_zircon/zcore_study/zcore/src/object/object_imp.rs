/// 空对象
use super::*; //为父模块的结构体进行方法实现，引入一个路径，省的在每个需要父类的地方都crate::object::
use spin::Mutex; //因为不能用依赖操作系统提供系统调用的std,所以用了no_std兼容的spin库中的mutex来实现简单互斥锁。具体实现细节先不深纠，一样用。
#[derive(Debug)] //属性宏，自动为结构体或枚举派生（derive） Debug trait，简单的说就是方便格式化打印用的
pub struct DummyObject {
    //dummy有填充物，哑巴之类的意思，dummyobject就是等待填充啥也干不了的空对象
    id: KoID,
    inner: Mutex<DummyObjectInner>, //利用一个带互斥锁的内部可变结构体来存放这个对象可变的成员
}

/// `DummyObject` 的内部可变部分
/// Mutex 会用最简单的方式帮我们处理好并发访问问题：如果有其他人正在访问，我就在这里忙等。
///  数据被 Mutex 包起来之后需要首先使用 lock() 拿到锁之后才能访问。
/// 此时并发访问已经安全，因此被包起来的结构自动具有了 Send + Sync 特性。
///
/// 疑惑：将所有可变成员用一个互斥锁管理，真的合理吗？
/// 使用单个 Mutex 来保护所有可变成员确实可以简化并发访问控制，但这样做可能会降低性能。因为每次只有一个线程可以修改数据，即使这些修改是独立的且不会相互冲突。
/// 当然啦,具体情况还未知，先这样做吧！
#[derive(Default, Debug)] //自动派生deault和debug,以产生默认实例和格式化打印。
struct DummyObjectInner {
    name: String, //内核对象名
}

use alloc::sync::Arc; //Atomic Reference Counting 引入原子引用计数智能指针。
use core::sync::atomic::*; //引入core库的一组原子操作
                           //尽管alloce和core都有sync模块,但他们有所不同。core::sync::atomic 提供了一组原子操作，而 alloc::sync 提供了一些更高级的同步原语，这些原语可能需要额外的内存分配功能。

impl DummyObject {
    /// 创建一个新 `DummyObject`
    pub fn new() -> Arc<Self> {
        //返回一个指向self的原子引用计数指针。
        Arc::new(DummyObject {
            id: Self::new_koid(),      //调用new_koid生成唯一id
            inner: Default::default(), //调用default宏生成默认的所有可变内部成员
        })
    }

    /// 生成一个唯一的 ID
    fn new_koid() -> KoID {
        //声明了一个静态变量 NEXT_KOID，其类型为 AtomicU64，初始值为 1024。
        //AtomicU64 是一个原子操作的无符号 64 位整数，它保证了多线程环境下对这个变量的访问是安全的。
        //静态变量在程序的生命周期内只初始化一次，并且可以在多个线程之间共享，
        static NEXT_KOID: AtomicU64 = AtomicU64::new(1024);
        //返回next_koid,并在”返回后“自增1，从这里生成的id也就是唯一的了。
        NEXT_KOID.fetch_add(1, Ordering::SeqCst)
    }
}

//为空对象实现”内核对象“特性
impl KernelObject for DummyObject {
    fn id(&self) -> KoID {
        self.id
    }
    fn type_name(&self) -> &str {
        "DummyObject"
    }
    fn name(&self) -> String {
        self.inner.lock().name.clone() //取得内部互斥锁，访问name,返回一个拷贝。
    }
    fn set_name(&self, name: &str) {
        //获取空对象的不可变引用（这里是为了保护不可变的部分）
        //可以像访问任何结构体的字段一样直接访问lock函数返回的 MutexGuard<T> 的字段，而不需要先解引用整个 MutexGuard 对象
        self.inner.lock().name = String::from(name); //通过lock()取得内部互斥锁的可变访问权，利用传入的参数修改name。
    }
}
