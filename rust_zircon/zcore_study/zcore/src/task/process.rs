use spin::Mutex; //因为不能用依赖操作系统提供系统调用的std,所以用了no_std兼容的spin库中的mutex来实现简单互斥锁。具体实现细节先不深纠，一样用。
use alloc::collections::BTreeMap;
use alloc::sync::Arc;



use crate::object::*; //引入object模块（包括父模块和子模块，因为在父模块中公开引入了所有子模块，所以在这里只要*就可以了）
use crate::impl_kobject;  //虽然impl_kobject是在object模块下实现的，但#[macro_export] 导出宏到crate根了，所以要从crate里引入。

#[allow(dead_code)]
/// 进程对象
pub struct Process {
    base: KObjectBase,                 //注意：基类中也有一个inner,里面保存的是基类的可变部分。
    inner: Mutex<ProcessInner>,        //这里是进程对象的可变部分
}
impl_kobject!(Process);// 宏的作用：补充
#[allow(dead_code)]
struct ProcessInner {
    handles: BTreeMap<HandleValue, Handle>, //进程对象的内部可变部分是一个用BTreeMap实现的句柄， 用于构建树的key是HandleValue，value就是句柄
}

pub type HandleValue = u32; //在这定义一个类型用作键值对中的key

impl Process {
    /// 创建一个新的进程对象
    pub fn new() -> Arc<Self> {
        Arc::new(Process {
            base: KObjectBase::default(),
            inner: Mutex::new(ProcessInner {
                handles: BTreeMap::default(), //创建一个空的B树，或者B+树？不重要，具体实现不追究了，总之是一种键值对的存储方式。
            }),
        })
    }
    ///为调用此函数的进程对象添加一个句柄
    pub fn add_handle(&self, handle: Handle) -> HandleValue {

        let mut inner = self.inner.lock();  //取得锁
        //从0开始找一个当前树中没有的索引作为key（handle_value）返回
        let value = (0 as HandleValue..)   
            .find(|idx| !inner.handles.contains_key(idx))
            .unwrap();
        // 插入BTreeMap
        inner.handles.insert(value, handle);
        value
    }
    ///传入作为key的handlevalue,删除对应句柄
    pub fn remove_handle(&self, handle_value: HandleValue) {
        self.inner.lock().handles.remove(&handle_value);
    }
    /// 根据句柄值查找内核对象，并检查权限
    pub fn get_object_with_rights<T: KernelObject>(
        &self,
        handle_value: HandleValue,
        desired_rights: Rights,
    ) -> ZxResult<Arc<T>> {
        let handle = self
            .inner
            .lock()     //取锁以访问句柄树
            .handles    
            .get(&handle_value)          //根据key在process对象的句柄树中查找对应的句柄对象
            .ok_or(ZxError::BAD_HANDLE)?        //如果找不到对应的句柄，函数将返回一个 ZxError::BAD_HANDLE 错误
            .clone();            //使用 .clone() 方法克隆句柄对象，以便在不改变原始集合的情况下使用句柄。
        // check type before rights
        let object = handle          //利用这个句柄对象
            .object            //找到它对应的抽象对象
            .downcast_arc::<T>()                 //向下转换成被Arc包裹的具体类型对象
            .map_err(|_| ZxError::WRONG_TYPE)?;  //闭包 |_| ZxError::WRONG_TYPE 表示不管原始错误是什么，都将其转换为 ZxError::WRONG_TYPE
        if !handle.rights.contains(desired_rights) { //如果这个句柄不包含应有的权限
            return Err(ZxError::ACCESS_DENIED);   //报错，访问被拒绝
        }
        Ok(object) //一切正常后，返回一个对“要查找对象”的Arc克隆。
    }
}

///自定义错误类型，之后可能会把他移到其他位置，不过目前就在这吧
pub type ZxResult<T> = Result<T, ZxError>;
#[allow(non_camel_case_types, dead_code)]
#[repr(i32)]
#[derive(Debug, Clone, Copy)]
pub enum ZxError {
    OK = 0,
    /// 一个不指向handle的特定的handle value
    BAD_HANDLE = -11,
    /// 操作主体对于执行这个操作来说是错误的类型
    /// 例如： 尝试执行 message_read 在 thread handle.
    WRONG_TYPE = -12,
    // 权限检查错误
    // 调用者没有执行该操作的权限
    ACCESS_DENIED = -30,
}


#[cfg(test)]
mod process_test{
    use super::*;
    use alloc::format;
    #[test]    
    ///测试进程对象的各个功能是否正常
    fn new_proc() {
        let proc = Process::new();
        assert_eq!(proc.type_name(), "Process");
        assert_eq!(proc.name(), "");
        proc.set_name("proc1");
        assert_eq!(proc.name(), "proc1");
        assert_eq!(
            format!("{:?}", proc),
            format!("Process({}, \"proc1\")", proc.id())
        );

        let obj: Arc<dyn KernelObject> = proc;
        assert_eq!(obj.type_name(), "Process");
        assert_eq!(obj.name(), "proc1");
        obj.set_name("proc2");
        assert_eq!(obj.name(), "proc2");
        assert_eq!(
            format!("{:?}", obj),
            format!("Process({}, \"proc2\")", obj.id())
        );
    }
    #[test]    
    fn proc_handle() {
        let proc = Process::new();
        let handle = Handle::new(proc.clone(), Rights::DEFAULT_PROCESS); //创建一个包含”默认进程“权限，连接到proc对象的句柄
        let handle_value = proc.add_handle(handle); //将句柄授予进程，并用handle_value保存此句柄的key
        //这里利用key找到handle句柄，并检查其权限，最后返回proc对象，让object1共享其所有权。
        let object1: Arc<Process> = proc 
            .get_object_with_rights(handle_value, Rights::DEFAULT_PROCESS)
            .expect("failed to get object");
        //断言他们由一个arc所管理，指向同样的实例
        assert!(Arc::ptr_eq(&object1, &proc));

        proc.remove_handle(handle_value);
    }
}
