use alloc::string::String; //用不了std，所以用alloc提供的String
pub mod object_imp; //获取访问子模块的权限，在这个子模块中对object进行实现（implement）
/// 内核对象公共接口，要求所有的内核对象都实现这样的trait。
/// 任何实现了 KernelObject 的类型都必须同时满足 Send 和 Sync 这两个标记 trait，以保证其可以安全的被多线程共享访问。
/// Send trait 表示某个类型可以安全地在多线程之间发送。这意味着类型的实例可以安全地从一个线程的栈或静态存储区移动到另一个线程。
/// Sync trait 表示多个线程可以同时访问某个类型的实例，而不会有数据竞争的风险。这意味着类型的实例可以安全地被多个线程引用。
/// 内核对象可能在多个线程间传递，且可能被多个线程同时访问，因此它必须是并发对象，所以要实现这两个这个trait。
pub trait KernelObject: Send + Sync {
    /// 获取对象 ID
    fn id(&self) -> KoID;
    /// 获取对象类型名
    fn type_name(&self) -> &str; //如果返回的字符串是静态的或者生命周期足够长，可以使用 &str。
    /// 获取对象名称
    fn name(&self) -> String; //如果需要返回一个动态生成的、可以独立于原始数据存在的字符串副本，或者需要保证字符串的可变性，那么使用 String 更合适。
    /// 设置对象名称
    fn set_name(&self, name: &str);
}

/// 对象 ID 类型
pub type KoID = u64; //kernel_object ID
