///自定义错误类型，为方便处理各个模块的各种错误，将他专门拿出来做一个模块
pub type ZxResult<T> = Result<T, ZxError>;
#[allow(non_camel_case_types, dead_code)]
#[repr(i32)]
#[derive(Debug, Clone, Copy)]
#[derive(PartialEq)]   //为了方便断言错误返回
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
    //对端已经关闭
    PEER_CLOSED=-13,
    //需要等待
    SHOULD_WAIT=-14,
}
