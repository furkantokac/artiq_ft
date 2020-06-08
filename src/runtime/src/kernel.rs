use core::{ptr, mem};
use log::{debug, error};
use alloc::{vec::Vec, sync::Arc};
use cslice::CSlice;

use libcortex_a9::{mutex::Mutex, sync_channel::{self, sync_channel}};
use libsupport_zynq::boot::Core1;

use dyld;
use crate::rpc;
use crate::rtio;


#[derive(Debug)]
pub enum Message {
    LoadRequest(Arc<Vec<u8>>),
    LoadCompleted,
    LoadFailed,
    StartRequest,
    KernelFinished,
    RpcSend { is_async: bool, data: Arc<Vec<u8>> },
    RpcRecvRequest(*mut ()),
    RpcRecvReply(Result<usize, ()>),
}

static CHANNEL_0TO1: Mutex<Option<sync_channel::Receiver<Message>>> = Mutex::new(None);
static CHANNEL_1TO0: Mutex<Option<sync_channel::Sender<Message>>> = Mutex::new(None);

pub struct Control {
    core1: Core1,
    pub tx: sync_channel::Sender<Message>,
    pub rx: sync_channel::Receiver<Message>,
}

impl Control {
    pub fn start() -> Self {
        let core1 = Core1::start(true);

        let (core0_tx, core1_rx) = sync_channel(4);
        let (core1_tx, core0_rx) = sync_channel(4);
        *CHANNEL_0TO1.lock() = Some(core1_rx);
        *CHANNEL_1TO0.lock() = Some(core1_tx);

        Control {
            core1,
            tx: core0_tx,
            rx: core0_rx,
        }
    }

    pub fn restart(&mut self) {
        *CHANNEL_0TO1.lock() = None;
        *CHANNEL_1TO0.lock() = None;

        self.core1.restart();

        let (core0_tx, core1_rx) = sync_channel(4);
        let (core1_tx, core0_rx) = sync_channel(4);
        *CHANNEL_0TO1.lock() = Some(core1_rx);
        *CHANNEL_1TO0.lock() = Some(core1_tx);
        self.tx = core0_tx;
        self.rx = core0_rx;
    }
}

static mut KERNEL_CHANNEL_0TO1: *mut () = ptr::null_mut();
static mut KERNEL_CHANNEL_1TO0: *mut () = ptr::null_mut();

fn rpc_send_common(is_async: bool, service: u32, tag: &CSlice<u8>, data: *const *const ()) {
    let core1_tx: &mut sync_channel::Sender<Message> = unsafe { mem::transmute(KERNEL_CHANNEL_1TO0) };
    let mut buffer = Vec::<u8>::new();
    rpc::send_args(&mut buffer, service, tag.as_ref(), data).expect("RPC encoding failed");
    core1_tx.send(Message::RpcSend { is_async: is_async, data: Arc::new(buffer) });
}

extern fn rpc_send(service: u32, tag: &CSlice<u8>, data: *const *const ()) {
    rpc_send_common(false, service, tag, data);
}

extern fn rpc_send_async(service: u32, tag: &CSlice<u8>, data: *const *const ()) {
    rpc_send_common(true, service, tag, data);
}

extern fn rpc_recv(slot: *mut ()) -> usize {
    let core1_rx: &mut sync_channel::Receiver<Message> = unsafe { mem::transmute(KERNEL_CHANNEL_0TO1) };
    let core1_tx: &mut sync_channel::Sender<Message> = unsafe { mem::transmute(KERNEL_CHANNEL_1TO0) };
    core1_tx.send(Message::RpcRecvRequest(slot));
    let reply = core1_rx.recv();
    match *reply {
        Message::RpcRecvReply(Ok(alloc_size)) => alloc_size,
        Message::RpcRecvReply(Err(_)) => unimplemented!(),
        _ => panic!("received unexpected reply to RpcRecvRequest: {:?}", reply)
    }
}

macro_rules! api {
    ($i:ident) => ({
        extern { static $i: u8; }
        api!($i = &$i as *const _)
    });
    ($i:ident, $d:item) => ({
        $d
        api!($i = $i)
    });
    ($i:ident = $e:expr) => {
        (stringify!($i), $e as *const ())
    }
}

fn resolve(required: &[u8]) -> Option<u32> {
    let api = &[
        api!(now_mu = rtio::now_mu),
        api!(at_mu = rtio::at_mu),
        api!(delay_mu = rtio::delay_mu),

        api!(rpc_send = rpc_send),
        api!(rpc_send_async = rpc_send_async),
        api!(rpc_recv = rpc_recv),

        api!(rtio_init = rtio::init),
        api!(rtio_get_destination_status = rtio::get_destination_status),
        api!(rtio_get_counter = rtio::get_counter),
        api!(rtio_output = rtio::output),
        api!(rtio_output_wide = rtio::output_wide),
        api!(rtio_input_timestamp = rtio::input_timestamp),
        api!(rtio_input_data = rtio::input_data),
        api!(rtio_input_timestamped_data = rtio::input_timestamped_data),

        api!(__artiq_personality = 0), // HACK
    ];
    api.iter()
       .find(|&&(exported, _)| exported.as_bytes() == required)
       .map(|&(_, ptr)| ptr as u32)
}


#[no_mangle]
pub fn main_core1() {
    debug!("Core1 started");

    unsafe {
        llvm_asm!("
            mrc p15, 0, r1, c1, c0, 2
            orr r1, r1, (0b1111<<20)
            mcr p15, 0, r1, c1, c0, 2

            vmrs r1, fpexc
            orr r1, r1, (1<<30)
            vmsr fpexc, r1
            ":::"r1");
    }
    debug!("FPU enabled on Core1");

    let mut core1_tx = None;
    while core1_tx.is_none() {
        core1_tx = CHANNEL_1TO0.lock().take();
    }
    let mut core1_tx = core1_tx.unwrap();

    let mut core1_rx = None;
    while core1_rx.is_none() {
        core1_rx = CHANNEL_0TO1.lock().take();
    }
    let mut core1_rx = core1_rx.unwrap();

    let mut current_modinit: Option<u32> = None;
    loop {
        let message = core1_rx.recv();
        match *message {
            Message::LoadRequest(data) => {
                match dyld::load(&data, &resolve) {
                    Ok(library) => {
                        let bss_start = library.lookup(b"__bss_start");
                        let end = library.lookup(b"_end");
                        if let Some(bss_start) = bss_start {
                            let end = end.unwrap();
                            unsafe {
                                ptr::write_bytes(bss_start as *mut u8, 0, (end - bss_start) as usize);
                            }
                        }
                        let __modinit__ = library.lookup(b"__modinit__").unwrap();
                        current_modinit = Some(__modinit__);
                        debug!("kernel loaded");
                        core1_tx.send(Message::LoadCompleted);
                    },
                    Err(error) => {
                        error!("failed to load shared library: {}", error);
                        core1_tx.send(Message::LoadFailed);
                    }
                }
            },
            Message::StartRequest => {
                debug!("kernel starting");
                if let Some(__modinit__) = current_modinit {
                    unsafe {
                        KERNEL_CHANNEL_0TO1 = mem::transmute(&mut core1_rx);
                        KERNEL_CHANNEL_1TO0 = mem::transmute(&mut core1_tx);
                        (mem::transmute::<u32, fn()>(__modinit__))();
                        KERNEL_CHANNEL_0TO1 = ptr::null_mut();
                        KERNEL_CHANNEL_1TO0 = ptr::null_mut();
                    }
                }
                debug!("kernel finished");
                core1_tx.send(Message::KernelFinished);
            }
            _ => error!("Core1 received unexpected message: {:?}", message),
        }
    }
}
