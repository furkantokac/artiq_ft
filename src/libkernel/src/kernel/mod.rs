use core::ptr;

use libcortex_a9::{mutex::Mutex, semaphore::Semaphore, sync_channel};

use crate::Message;

mod control;
pub use control::Control;
mod api;
pub mod core1;
mod dma;
mod rpc;
pub use dma::DmaRecorder;
mod cache;
#[cfg(has_drtio)]
mod subkernel;

static CHANNEL_0TO1: Mutex<Option<sync_channel::Sender<'static, Message>>> = Mutex::new(None);
static CHANNEL_1TO0: Mutex<Option<sync_channel::Receiver<'static, Message>>> = Mutex::new(None);
static CHANNEL_SEM: Semaphore = Semaphore::new(0, 1);

static mut KERNEL_CHANNEL_0TO1: Option<sync_channel::Receiver<'static, Message>> = None;
static mut KERNEL_CHANNEL_1TO0: Option<sync_channel::Sender<'static, Message>> = None;

pub static mut KERNEL_IMAGE: *const core1::KernelImage = ptr::null();

static INIT_LOCK: Mutex<()> = Mutex::new(());
