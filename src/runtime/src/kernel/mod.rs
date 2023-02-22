use alloc::{string::String, vec::Vec};
use core::ptr;

use libcortex_a9::{mutex::Mutex, semaphore::Semaphore, sync_channel};

use crate::eh_artiq;

mod control;
pub use control::Control;
mod api;
pub mod core1;
mod dma;
mod rpc;
pub use dma::DmaRecorder;
mod cache;

#[derive(Debug, Clone)]
pub struct RPCException {
    pub id: u32,
    pub message: u32,
    pub param: [i64; 3],
    pub file: u32,
    pub line: i32,
    pub column: i32,
    pub function: u32,
}

#[derive(Debug, Clone)]
pub enum Message {
    LoadRequest(Vec<u8>),
    LoadCompleted,
    LoadFailed,
    StartRequest,
    KernelFinished(u8),
    KernelException(
        &'static [Option<eh_artiq::Exception<'static>>],
        &'static [eh_artiq::StackPointerBacktrace],
        &'static [(usize, usize)],
        u8,
    ),
    RpcSend {
        is_async: bool,
        data: Vec<u8>,
    },
    RpcRecvRequest(*mut ()),
    RpcRecvReply(Result<usize, RPCException>),

    CacheGetRequest(String),
    CacheGetReply(Vec<i32>),
    CachePutRequest(String, Vec<i32>),

    DmaPutRequest(DmaRecorder),
    DmaEraseRequest(String),
    DmaGetRequest(String),
    DmaGetReply(Option<(Vec<u8>, i64)>),

    #[cfg(has_drtio)]
    UpDestinationsRequest(i32),
    #[cfg(has_drtio)]
    UpDestinationsReply(bool),
}

static CHANNEL_0TO1: Mutex<Option<sync_channel::Sender<'static, Message>>> = Mutex::new(None);
static CHANNEL_1TO0: Mutex<Option<sync_channel::Receiver<'static, Message>>> = Mutex::new(None);
static CHANNEL_SEM: Semaphore = Semaphore::new(0, 1);

static mut KERNEL_CHANNEL_0TO1: Option<sync_channel::Receiver<'static, Message>> = None;
static mut KERNEL_CHANNEL_1TO0: Option<sync_channel::Sender<'static, Message>> = None;

pub static mut KERNEL_IMAGE: *const core1::KernelImage = ptr::null();

static INIT_LOCK: Mutex<()> = Mutex::new(());
