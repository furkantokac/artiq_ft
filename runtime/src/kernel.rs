use core::{ptr, mem};
use log::{debug, error};
use alloc::{vec, vec::Vec, sync::Arc};

use libcortex_a9::{mutex::Mutex, sync_channel::{self, sync_channel}};
use libsupport_zynq::boot::Core1;

use dyld;
use crate::rtio;


#[derive(Debug)]
pub enum Message {
    LoadRequest(Arc<Vec<u8>>),
    LoadCompleted,
    LoadFailed,
    StartRequest,
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
        let core1 = Core1::start();

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

    let mut core1_tx = None;
    while core1_tx.is_none() {
        core1_tx = CHANNEL_1TO0.lock().take();
    }
    let mut core1_tx = core1_tx.unwrap();

    let mut core1_rx = None;
    while core1_rx.is_none() {
        core1_rx = CHANNEL_0TO1.lock().take();
    }
    let core1_rx = core1_rx.unwrap();

    let mut image = vec![0; 1024*1024];
    let mut current_modinit: Option<u32> = None;
    for message in core1_rx {
        match *message {
            Message::LoadRequest(data) => {
                match dyld::Library::load(&data, &mut image, &resolve) {
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
                        core1_tx.send(Message::LoadCompleted)
                    },
                    Err(error) => {
                        error!("failed to load shared library: {}", error);
                        core1_tx.send(Message::LoadFailed)
                    }
                }
            },
            Message::StartRequest => {
                debug!("kernel starting");
                if let Some(__modinit__) = current_modinit {
                    unsafe {
                        (mem::transmute::<u32, fn()>(__modinit__))();
                    }
                }
                debug!("kernel terminated");
            }
            _ => error!("Core1 received unexpected message: {:?}", message),
        }
    }
}
