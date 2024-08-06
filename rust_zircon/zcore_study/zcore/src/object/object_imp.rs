/// 空对象
use super::*; //为父模块的结构体进行方法实现，引入一个路径，省的在每个需要父类的地方都crate::object::
use spin::Mutex; //因为不能用依赖操作系统提供系统调用的std,所以用了no_std兼容的spin库中的mutex来实现简单互斥锁。具体实现细节先不深纠，一样用。
use alloc::sync::Arc;//原子引用计数，用于在多线程环境下安全的共享所有权
use core::sync::atomic::*;

pub struct KObjectBase {
    //dummy有填充物，哑巴之类的意思，dummyobject就是等待填充啥也干不了的空对象，在实现模拟继承后，由KObjectBase代替
    id: KoID,
    inner: Mutex<KObjectBaseInner>, //利用一个带互斥锁的内部可变结构体来存放这个对象可变的成员
}

/// `DummyObject` 的内部可变部分
/// Mutex 会用最简单的方式帮我们处理好并发访问问题：如果有其他人正在访问，我就在这里忙等。
///  数据被 Mutex 包起来之后需要首先使用 lock() 拿到锁之后才能访问。
/// 此时并发访问已经安全，因此被包起来的结构自动具有了 Send + Sync 特性。
///
/// 疑惑：将所有可变成员用一个互斥锁管理，真的合理吗？
/// 使用单个 Mutex 来保护所有可变成员确实可以简化并发访问控制，但这样做可能会降低性能。因为每次只有一个线程可以修改数据，即使这些修改是独立的且不会相互冲突。
/// 当然啦,具体情况还未知，先这样做吧！
#[derive(Default)] //自动派生deault,以产生默认实例
struct KObjectBaseInner {
    name: String, //内核对象名
}

impl Default for KObjectBase {
    /// 创建一个新 `KObjectBase`
    fn default() -> Self {
        KObjectBase {
            id: Self::new_koid(),
            inner: Default::default(),
        }
    }
}

//id自增
impl KObjectBase {
    fn new_koid() -> KoID {
        static NEXT_KOID: AtomicU64 = AtomicU64::new(1024);  //设置一个原子静态数据作为id计数器
        NEXT_KOID.fetch_add(1, Ordering::SeqCst)  //每次用完自增
    }
}

//为对象基类实现”内核对象“特性的相关函数，用于宏自动实现
impl KObjectBase {
    fn id(&self) -> KoID {
        self.id
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

/// 为内核对象 struct 自动实现 `KernelObject` trait 的宏。
#[macro_export] // 导出宏，可在 crate 外部使用
macro_rules! impl_kobject { //定义了一个名为 impl_kobject 的宏
    // 匹配类型名，并可以提供函数覆盖默认实现
    //$class:ident：这部分是宏的一个参数，$class 是参数的名称，而 :ident 表示这个参数应该是一个标识符。
    // $fn:tt：这是宏的另一个参数，$fn 是参数的名称，:tt（token tree）表示这个参数可以是任何有效的 Rust 代码片段。
    //这意味着可以传递一系列 Rust 代码作为参数，这些代码可以是表达式、语句、甚至是模式。
    ($class:ident $( $fn:tt )*) => { 

        // 为对象实现 KernelObject trait，方法直接转发到内部 struct
        impl KernelObject for $class {
            fn id(&self) -> KoID {
                // 直接访问内部的 pub 属性
                self.base.id
            }
            fn type_name(&self) -> &str {
                // 用 stringify! 宏将输入的类型名转成字符串
                stringify!($class)
            }
            // 注意宏里面的类型要写完整路径，例如：alloc::string::String
            fn name(&self) -> alloc::string::String {
                self.base.name()
            }
            fn set_name(&self, name: &str){
                // 直接访问内部的 pub 方法
                self.base.set_name(name)
            }
            // 可以传入任意数量的函数，覆盖 trait 的默认实现
            //$( ... )* 是一个重复模式，表示括号内的代码可以出现零次或多次，直到宏定义的结束。
            //$fn 是一个宏的参数，它代表一个函数定义。在这里，它通常被期望是一个具体的函数实现，比如方法体。
            $( $fn )*
        }
        // 为对象实现 Debug trait，fmt是debug特性要求必须实现的函数
        impl core::fmt::Debug for $class {
            fn fmt(
                &self,
                f: &mut core::fmt::Formatter<'_>,
            ) -> core::result::Result<(), core::fmt::Error> {
                // 输出对象类型、ID 和名称
                f.debug_tuple(&stringify!($class))
                    .field(&self.id())
                    .field(&self.name())
                    .finish()
            }
        }
    };
}

/// 在实现了 trait 宏之后，用宏来定义一个空对象结构体
pub struct DummyObject {
    // 其中必须包含一个名为 `base` 的 `KObjectBase`
    base: KObjectBase,
}

// 使用刚才的宏，声明其为内核对象，自动生成必要的代码
impl_kobject!(DummyObject);

impl DummyObject {
    /// 创建一个新 `DummyObject`
    #[allow(dead_code)]
    pub fn new() -> Arc<Self> { //内核对象可能被多处引用，arc保证多线程环境可安全共享所有权
        Arc::new(DummyObject {
            base: KObjectBase::default(),
        })
    }
}