//! Kernel prologue/epilogue that runs on the 2nd CPU core

use core::{mem, ptr, cell::UnsafeCell};
use alloc::borrow::ToOwned;
use log::{debug, info, error};
use cslice::CSlice;

use libcortex_a9::{
    enable_fpu,
    cache::{dcci_slice, iciallu, bpiall},
    asm::{dsb, isb},
};
use dyld::{self, Library};
use crate::eh_artiq;
use super::{
    api::resolve,
    rpc::rpc_send_async,
    dma::init_dma,
    CHANNEL_0TO1, CHANNEL_1TO0,
    KERNEL_CHANNEL_0TO1, KERNEL_CHANNEL_1TO0,
    KERNEL_IMAGE,
    Message,
};

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

pub struct KernelImage {
    library: UnsafeCell<Library>,
    __modinit__: u32,
    typeinfo: Option<u32>,
}

impl KernelImage {
    pub fn new(library: Library) -> Result<Self, dyld::Error> {
        let __modinit__ = library.lookup(b"__modinit__")
            .ok_or(dyld::Error::Lookup("__modinit__".to_owned()))?;
        let typeinfo = library.lookup(b"typeinfo");

        // clear .bss
        let bss_start = library.lookup(b"__bss_start");
        let end = library.lookup(b"_end");
        if let Some(bss_start) = bss_start {
            let end = end
                .ok_or(dyld::Error::Lookup("_end".to_owned()))?;
            unsafe {
                ptr::write_bytes(bss_start as *mut u8, 0, (end - bss_start) as usize);
            }
        }

        Ok(KernelImage {
            library: UnsafeCell::new(library),
            __modinit__,
            typeinfo,
        })
    }

    pub unsafe fn rebind(&self, name: &[u8], addr: *const ()) -> Result<(), dyld::Error> {
        let library = self.library.get().as_mut().unwrap();
        library.rebind(name, addr)
    }

    pub unsafe fn exec(&self) {
        // Flush data cache entries for the image in DDR, including
        // Memory/Instruction Synchronization Barriers
        dcci_slice(self.library.get().as_ref().unwrap().image.data);
        iciallu();
        bpiall();
        dsb();
        isb();

        (mem::transmute::<u32, fn()>(self.__modinit__))();

        if let Some(typeinfo) = self.typeinfo {
            attribute_writeback(typeinfo as *const ());
        }
    }

    pub fn get_load_addr(&self) -> usize {
        unsafe {
            self.library.get().as_ref().unwrap().image.as_ptr() as usize
        }
    }
}

#[no_mangle]
pub fn main_core1() {
    debug!("Core1 started");

    enable_fpu();
    debug!("FPU enabled on Core1");

    init_dma();
    debug!("Init DMA!");

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

    // set on load, cleared on start
    let mut loaded_kernel = None;
    loop {
        let message = core1_rx.recv();
        match *message {
            Message::LoadRequest(data) => {
                let result = dyld::load(&data, &resolve)
                    .and_then(KernelImage::new);
                match result {
                    Ok(kernel) => {
                        loaded_kernel = Some(kernel);
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
                info!("kernel starting");
                if let Some(kernel) = loaded_kernel.take() {
                    core::mem::replace(&mut *KERNEL_CHANNEL_0TO1.lock(), Some(core1_rx));
                    core::mem::replace(&mut *KERNEL_CHANNEL_1TO0.lock(), Some(core1_tx));
                    unsafe {
                        KERNEL_IMAGE = &kernel as *const KernelImage;
                        kernel.exec();
                        KERNEL_IMAGE = ptr::null();
                    }
                    core1_rx = core::mem::replace(&mut *KERNEL_CHANNEL_0TO1.lock(), None).unwrap();
                    core1_tx = core::mem::replace(&mut *KERNEL_CHANNEL_1TO0.lock(), None).unwrap();
                }
                info!("kernel finished");
                core1_tx.send(Message::KernelFinished);
            }
            _ => error!("Core1 received unexpected message: {:?}", message),
        }
    }
}

/// Called by eh_artiq
pub fn terminate(exception: &'static eh_artiq::Exception<'static>, backtrace: &'static mut [usize]) -> ! {
    let load_addr = unsafe {
        KERNEL_IMAGE.as_ref().unwrap().get_load_addr()
    };
    let mut cursor = 0;
    // The address in the backtrace is relocated, so we have to convert it back to the address in
    // the original python script, and remove those Rust function backtrace.
    for i in 0..backtrace.len() {
        if backtrace[i] >= load_addr {
            backtrace[cursor] = backtrace[i] - load_addr;
            cursor += 1;
        }
    }

    {
        let mut core1_tx = KERNEL_CHANNEL_1TO0.lock();
        core1_tx.as_mut().unwrap().send(Message::KernelException(exception, &backtrace[..cursor]));
    }
    // TODO: remove after implementing graceful kernel termination.
    error!("Core1 uncaught exception");
    loop {}
}
