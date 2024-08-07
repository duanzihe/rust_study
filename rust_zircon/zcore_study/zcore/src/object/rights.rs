use bitflags::bitflags;


bitflags! {
    /// 句柄权限,借助 bitflags! 将一个 u32 的 rights 包装为一个 Rights 结构体
    pub struct Rights: u32 {
        const DUPLICATE = 1 << 0;
        const TRANSFER = 1 << 1;
        const READ = 1 << 2;
        const WRITE = 1 << 3;
        const EXECUTE = 1 << 4;
        const BASIC = 1 << 5;
        const DEFAULT_PROCESS = 1 << 5;

    }
}