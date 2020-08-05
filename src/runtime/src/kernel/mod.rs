use core::ptr;
use alloc::{vec::Vec, string::String};

use libcortex_a9::{mutex::Mutex, sync_channel};
use crate::eh_artiq;

mod control;
pub use control::Control;
pub mod core1;
mod api;
mod rpc;
mod dma;
pub use dma::DmaRecorder;
mod cache;

#[derive(Debug, Clone)]
pub struct RPCException {
    pub name: String,
    pub message: String,
    pub param: [i64; 3],
    pub file: String,
    pub line: i32,
    pub column: i32,
    pub function: String
}

#[derive(Debug, Clone)]
pub enum Message {
    LoadRequest(Vec<u8>),
    LoadCompleted,
    LoadFailed,
    StartRequest,
    KernelFinished,
    KernelException(&'static eh_artiq::Exception<'static>, &'static [usize]),
    RpcSend { is_async: bool, data: Vec<u8> },
    RpcRecvRequest(*mut ()),
    RpcRecvReply(Result<usize, RPCException>),

    CacheGetRequest(String),
    CacheGetReply(Vec<i32>),
    CachePutRequest(String, Vec<i32>),

    DmaPutRequest(DmaRecorder),
    DmaEraseRequest(String),
    DmaGetRequest(String),
    DmaGetReply(Option<(Vec<u8>, i64)>),
}

static CHANNEL_0TO1: Mutex<Option<sync_channel::Sender<'static, Message>>> = Mutex::new(None);
static CHANNEL_1TO0: Mutex<Option<sync_channel::Receiver<'static, Message>>> = Mutex::new(None);

static KERNEL_CHANNEL_0TO1: Mutex<Option<sync_channel::Receiver<'static, Message>>> = Mutex::new(None);
static KERNEL_CHANNEL_1TO0: Mutex<Option<sync_channel::Sender<'static, Message>>> = Mutex::new(None);

static mut KERNEL_IMAGE: *const core1::KernelImage = ptr::null();

static INIT_LOCK: Mutex<()> = Mutex::new(());

