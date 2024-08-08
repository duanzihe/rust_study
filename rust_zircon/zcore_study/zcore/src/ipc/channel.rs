use {
    super::*,
    crate::error::*,
    crate::object::*,
    alloc::collections::VecDeque,
    alloc::sync::{Arc, Weak},
    spin::Mutex,
    alloc::vec::Vec
};
#[derive(Default)]
pub struct MessagePacket {
    /// message packet携带的数据data
    pub data: Vec<u8>,
    /// message packet携带的句柄Handle
    pub handles: Vec<Handle>,
}
#[allow(dead_code)]
///channel结构体，Channel是唯一一个能传递handle的IPC，注意，这里的channel其实更像是endpoint！
pub struct Channel {
    base: KObjectBase,
    ///peer代表当前端点所在管道的另一个端点，两端的结构体分别持有对方的Weak引用，也就是说，一旦有一端的channel不再被强引用，那么channel就会销毁。
    peer: Mutex<Weak<Channel>>, 
    ///接收端队列，为什么明明是vecdeque双端动态队列，却只有接受端？这是因为发送端其实就是peer,可以利用其直接将数据写到peer对应的channel的recv中。
    recv_queue: Mutex<VecDeque<TMes>>,   
}

type TMes = MessagePacket;

//先模拟继承基类
impl_kobject!(Channel);

#[allow(dead_code)]
//再单独实现方法
impl Channel{
    ///提升对另一端channel的引用级别并返回可供访问的强引用，失败就返回一个对端已关闭的错误代码
    fn peer(&self) -> ZxResult<Arc<dyn KernelObject>> {
        let peer = self.peer.lock().upgrade().ok_or(ZxError::PEER_CLOSED)?; 
        Ok(peer)
    }
    fn peer_closed(&self) -> bool {
        self.peer.lock().upgrade().is_none()
    }
    fn related_koid(&self) -> KoID {
        self.peer.lock().upgrade().map(|p| p.id()).unwrap_or(0)
    }


    #[allow(dead_code)]
    pub fn create() -> (Arc<Self>, Arc<Self>) {
        let channel0 = Arc::new(Channel {
            base: KObjectBase::default(),
            peer: Mutex::new(Weak::default()),
            recv_queue: Default::default(),
        });
        let channel1 = Arc::new(Channel {
            base: KObjectBase::default(),
            peer: Mutex::new(Arc::downgrade(&channel0)),
            recv_queue: Default::default(),
        });
        //今天忽然反应过来了，我另一边的channel1获取的是弱引用啊，弱引用又没在引用计数里，为什么不能用get_mut？
        //而且get_mut立刻就使用了获取的可变引用，也不影响引用计数啊，先这么试试。
        *channel0.peer.lock() = Arc::downgrade(&channel1);
        // no other reference of `channel0`
        // unsafe {
        //     //疑难：channel的创建问题
        //     Arc::get_mut_unchecked(&mut channel0).peer = Arc::downgrade(&channel1); 
        // }
        (channel0, channel1)
    }
    ///读,成功了返回一个message package也就是TMes。
    pub fn read(&self) -> ZxResult<TMes> {
        let mut recv_queue = self.recv_queue.lock();
        if let Some(_msg) = recv_queue.front() {
            let msg = recv_queue.pop_front().unwrap();
            return Ok(msg);
        }
        if self.peer_closed() {
            Err(ZxError::PEER_CLOSED)
        } else {
            Err(ZxError::SHOULD_WAIT)
        }
    }
    ///写,成功了返回一个空元组，将消息压入对端channel的队尾。
    pub fn write(&self, msg: TMes) -> ZxResult<()>{                     //注意，返回元组也是返回！也得用ZxResult处理一下。
        let peer = self.peer.lock().upgrade().ok_or(ZxError::PEER_CLOSED)?; //先利用peer获取一下对端的channel
        peer.push_general(msg); 
        Ok(())
    }
    ///将消息包压入队尾
fn push_general(&self, msg: TMes) {  
        let mut send_queue = self.recv_queue.lock();
        send_queue.push_back(msg); 
    }
}

#[cfg(test)]
mod channel_test{
    use super::*;
    #[test]
    fn test_basics() {
        let (end0, end1) = Channel::create();
        assert!(Arc::ptr_eq(
            &end0.peer().unwrap().downcast_arc().unwrap(),
            &end1
        ));
        assert_eq!(end0.related_koid(), end1.id());

        drop(end1);
        assert_eq!(end0.peer().unwrap_err(), ZxError::PEER_CLOSED);
        assert_eq!(end0.related_koid(), 0);
    }
    #[test]
    fn read_write() {
        let (channel0, channel1) = Channel::create();
        //彼此传递一个消息
        channel0
            .write(MessagePacket {
                data: Vec::from("hello 1"),
                handles: Vec::new(),
            })
            .unwrap();

        channel1
            .write(MessagePacket {
                data: Vec::from("hello 0"),
                handles: Vec::new(),
            })
            .unwrap();

        // 读一个消息应该成功
        let recv_msg = channel1.read().unwrap();
        assert_eq!(recv_msg.data.as_slice(), b"hello 1");
        assert!(recv_msg.handles.is_empty());

        let recv_msg = channel0.read().unwrap();
        assert_eq!(recv_msg.data.as_slice(), b"hello 0");
        assert!(recv_msg.handles.is_empty());

        // 读更多消息应该失败
        assert_eq!(channel0.read().err(), Some(ZxError::SHOULD_WAIT));
        assert_eq!(channel1.read().err(), Some(ZxError::SHOULD_WAIT));
    }
}