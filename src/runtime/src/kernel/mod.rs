use core::ptr;
use alloc::{vec::Vec, sync::Arc, string::String};

use libcortex_a9::{mutex::Mutex, sync_channel};
use crate::eh_artiq;

mod control;
pub use control::Control;
pub mod core1;
mod api;
mod rpc;
mod dma;
mod cache;

#[derive(Debug)]
pub struct RPCException {
    pub name: String,
    pub message: String,
    pub param: [i64; 3],
    pub file: String,
    pub line: i32,
    pub column: i32,
    pub function: String
}

#[derive(Debug)]
pub enum Message {
    LoadRequest(Arc<Vec<u8>>),
    LoadCompleted,
    LoadFailed,
    StartRequest,
    KernelFinished,
    KernelException(&'static eh_artiq::Exception<'static>, &'static [usize]),
    RpcSend { is_async: bool, data: Arc<Vec<u8>> },
    RpcRecvRequest(*mut ()),
    RpcRecvReply(Result<usize, RPCException>),
}

static CHANNEL_0TO1: Mutex<Option<sync_channel::Receiver<Message>>> = Mutex::new(None);
static CHANNEL_1TO0: Mutex<Option<sync_channel::Sender<Message>>> = Mutex::new(None);

static KERNEL_CHANNEL_0TO1: Mutex<Option<sync_channel::Receiver<Message>>> = Mutex::new(None);
static KERNEL_CHANNEL_1TO0: Mutex<Option<sync_channel::Sender<Message>>> = Mutex::new(None);

static mut KERNEL_IMAGE: *const core1::KernelImage = ptr::null();

