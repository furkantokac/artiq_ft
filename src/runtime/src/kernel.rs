use core::{ptr, mem};
use log::{debug, error};
use alloc::{vec::Vec, sync::Arc};
use cslice::CSlice;

use libcortex_a9::{cache::dcci_slice, mutex::Mutex, sync_channel::{self, sync_channel}};
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

unsafe fn attribute_writeback(typeinfo: *const ()) {
    struct Attr {
        offset: usize,
        tag:    CSlice<'static, u8>,
        name:   CSlice<'static, u8>
    }

    struct Type {
        attributes: *const *const Attr,
        objects:    *const *const ()
    }

    let mut tys = typeinfo as *const *const Type;
    while !(*tys).is_null() {
        let ty = *tys;
        tys = tys.offset(1);

        let mut objects = (*ty).objects;
        while !(*objects).is_null() {
            let object = *objects;
            objects = objects.offset(1);

            let mut attributes = (*ty).attributes;
            while !(*attributes).is_null() {
                let attribute = *attributes;
                attributes = attributes.offset(1);

                if (*attribute).tag.len() > 0 {
                    rpc_send_async(0, &(*attribute).tag, [
                        &object as *const _ as *const (),
                        &(*attribute).name as *const _ as *const (),
                        (object as usize + (*attribute).offset) as *const ()
                    ].as_ptr());
                }
            }
        }
    }
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

extern fn exception_unimplemented() {
    unimplemented!();
}

macro_rules! api {
    ($i:ident) => ({
        extern { static $i: u8; }
        unsafe { api!($i = &$i as *const _) }
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
        // timing
        api!(now_mu = rtio::now_mu),
        api!(at_mu = rtio::at_mu),
        api!(delay_mu = rtio::delay_mu),

        // rpc
        api!(rpc_send = rpc_send),
        api!(rpc_send_async = rpc_send_async),
        api!(rpc_recv = rpc_recv),

        // rtio
        api!(rtio_init = rtio::init),
        api!(rtio_get_destination_status = rtio::get_destination_status),
        api!(rtio_get_counter = rtio::get_counter),
        api!(rtio_output = rtio::output),
        api!(rtio_output_wide = rtio::output_wide),
        api!(rtio_input_timestamp = rtio::input_timestamp),
        api!(rtio_input_data = rtio::input_data),
        api!(rtio_input_timestamped_data = rtio::input_timestamped_data),

        // Double-precision floating-point arithmetic helper functions
        // RTABI chapter 4.1.2, Table 2
        api!(__aeabi_dadd),
        api!(__aeabi_ddiv),
        api!(__aeabi_dmul),
        api!(__aeabi_dsub),
        // Double-precision floating-point comparison helper functions
        // RTABI chapter 4.1.2, Table 3
        api!(__aeabi_dcmpeq),
        api!(__aeabi_dcmpeq),
        api!(__aeabi_dcmplt),
        api!(__aeabi_dcmple),
        api!(__aeabi_dcmpge),
        api!(__aeabi_dcmpgt),
        api!(__aeabi_dcmpun),
        // Single-precision floating-point arithmetic helper functions
        // RTABI chapter 4.1.2, Table 4
        api!(__aeabi_fadd),
        api!(__aeabi_fdiv),
        api!(__aeabi_fmul),
        api!(__aeabi_fsub),
        // Single-precision floating-point comparison helper functions
        // RTABI chapter 4.1.2, Table 5
        api!(__aeabi_fcmpeq),
        api!(__aeabi_fcmpeq),
        api!(__aeabi_fcmplt),
        api!(__aeabi_fcmple),
        api!(__aeabi_fcmpge),
        api!(__aeabi_fcmpgt),
        api!(__aeabi_fcmpun),
        // Floating-point to integer conversions.
        // RTABI chapter 4.1.2, Table 6
        api!(__aeabi_d2iz),
        api!(__aeabi_d2uiz),
        api!(__aeabi_d2lz),
        api!(__aeabi_d2ulz),
        api!(__aeabi_f2iz),
        api!(__aeabi_f2uiz),
        api!(__aeabi_f2lz),
        api!(__aeabi_f2ulz),
        // Conversions between floating types.
        // RTABI chapter 4.1.2, Table 7
        api!(__aeabi_f2d),
        // Integer to floating-point conversions.
        // RTABI chapter 4.1.2, Table 8
        api!(__aeabi_i2d),
        api!(__aeabi_ui2d),
        api!(__aeabi_l2d),
        api!(__aeabi_ul2d),
        api!(__aeabi_i2f),
        api!(__aeabi_ui2f),
        api!(__aeabi_l2f),
        api!(__aeabi_ul2f),
        // Long long helper functions
        // RTABI chapter 4.2, Table 9
        api!(__aeabi_lmul),
        api!(__aeabi_llsl),
        api!(__aeabi_llsr),
        api!(__aeabi_lasr),
        // Integer division functions
        // RTABI chapter 4.3.1
        api!(__aeabi_idiv),
        api!(__aeabi_ldivmod),
        api!(__aeabi_uidiv),
        api!(__aeabi_uldivmod),
        // 4.3.4 Memory copying, clearing, and setting
        api!(__aeabi_memcpy8),
        api!(__aeabi_memcpy4),
        api!(__aeabi_memcpy),
        api!(__aeabi_memmove8),
        api!(__aeabi_memmove4),
        api!(__aeabi_memmove),
        api!(__aeabi_memset8),
        api!(__aeabi_memset4),
        api!(__aeabi_memset),
        api!(__aeabi_memclr8),
        api!(__aeabi_memclr4),
        api!(__aeabi_memclr),

        // exceptions
        api!(_Unwind_Resume = exception_unimplemented),
        api!(__artiq_personality = exception_unimplemented),
        api!(__artiq_raise = exception_unimplemented),
        api!(__artiq_reraise = exception_unimplemented),

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
    let mut current_typeinfo: Option<u32> = None;
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
                        current_typeinfo = library.lookup(b"typeinfo");
                        debug!("kernel loaded");
                        // Flush data cache entries for the image in DDR, including
                        // Memory/Instruction Symchronization Barriers
                        dcci_slice(library.image.data);

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
                        if let Some(typeinfo) = current_typeinfo {
                            attribute_writeback(typeinfo as *const ());
                        }
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
