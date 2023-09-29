use alloc::{collections::{BTreeMap, VecDeque},
            format,
            string::{String, ToString},
            vec::Vec};
use core::{cmp::min, option::NoneError, slice, str};

use core_io::{Error as IoError, Write};
use cslice::AsCSlice;
use io::{Cursor, ProtoRead, ProtoWrite};
use ksupport::{eh_artiq, kernel, rpc};
use libboard_artiq::{drtioaux_proto::{MASTER_PAYLOAD_MAX_SIZE, SAT_PAYLOAD_MAX_SIZE},
                     pl::csr};
use libboard_zynq::{time::Milliseconds, timer::GlobalTimer};
use libcortex_a9::sync_channel::Receiver;
use log::warn;

#[derive(Debug, Clone, Copy, PartialEq)]
enum KernelState {
    Absent,
    Loaded,
    Running,
    MsgAwait(Milliseconds),
    MsgSending,
}

#[derive(Debug)]
pub enum Error {
    Load(String),
    KernelNotFound,
    Unexpected(String),
    NoMessage,
    AwaitingMessage,
    SubkernelIoError,
    KernelException(Sliceable),
}

impl From<NoneError> for Error {
    fn from(_: NoneError) -> Error {
        Error::KernelNotFound
    }
}

impl From<IoError> for Error {
    fn from(_value: IoError) -> Error {
        Error::SubkernelIoError
    }
}

impl From<()> for Error {
    fn from(_: ()) -> Error {
        Error::NoMessage
    }
}

macro_rules! unexpected {
    ($($arg:tt)*) => (return Err(Error::Unexpected(format!($($arg)*))));
}

/* represents data that has to be sent to Master */
#[derive(Debug)]
pub struct Sliceable {
    it: usize,
    data: Vec<u8>,
}

/* represents interkernel messages */
struct Message {
    count: u8,
    tag: u8,
    data: Vec<u8>,
}

#[derive(PartialEq)]
enum OutMessageState {
    NoMessage,
    MessageReady,
    MessageBeingSent,
    MessageSent,
    MessageAcknowledged,
}

/* for dealing with incoming and outgoing interkernel messages */
struct MessageManager {
    out_message: Option<Sliceable>,
    out_state: OutMessageState,
    in_queue: VecDeque<Message>,
    in_buffer: Option<Message>,
}

// Per-run state
struct Session {
    id: u32,
    kernel_state: KernelState,
    last_exception: Option<Sliceable>,
    messages: MessageManager,
}

impl Session {
    pub fn new(id: u32) -> Session {
        Session {
            id: id,
            kernel_state: KernelState::Absent,
            last_exception: None,
            messages: MessageManager::new(),
        }
    }

    fn running(&self) -> bool {
        match self.kernel_state {
            KernelState::Absent | KernelState::Loaded => false,
            KernelState::Running | KernelState::MsgAwait { .. } | KernelState::MsgSending => true,
        }
    }
}

#[derive(Debug)]
struct KernelLibrary {
    library: Vec<u8>,
    complete: bool,
}

pub struct Manager<'a> {
    kernels: BTreeMap<u32, KernelLibrary>,
    session: Session,
    control: &'a mut kernel::Control,
    cache: BTreeMap<String, Vec<i32>>,
    last_finished: Option<SubkernelFinished>,
}

pub struct SubkernelFinished {
    pub id: u32,
    pub with_exception: bool,
}

pub struct SliceMeta {
    pub len: u16,
    pub last: bool,
}

macro_rules! get_slice_fn {
    ($name:tt, $size:expr) => {
        pub fn $name(&mut self, data_slice: &mut [u8; $size]) -> SliceMeta {
            if self.data.len() == 0 {
                return SliceMeta { len: 0, last: true };
            }
            let len = min($size, self.data.len() - self.it);
            let last = self.it + len == self.data.len();

            data_slice[..len].clone_from_slice(&self.data[self.it..self.it + len]);
            self.it += len;

            SliceMeta {
                len: len as u16,
                last: last,
            }
        }
    };
}

impl Sliceable {
    pub fn new(data: Vec<u8>) -> Sliceable {
        Sliceable { it: 0, data: data }
    }

    get_slice_fn!(get_slice_sat, SAT_PAYLOAD_MAX_SIZE);
    get_slice_fn!(get_slice_master, MASTER_PAYLOAD_MAX_SIZE);
}

impl MessageManager {
    pub fn new() -> MessageManager {
        MessageManager {
            out_message: None,
            out_state: OutMessageState::NoMessage,
            in_queue: VecDeque::new(),
            in_buffer: None,
        }
    }

    pub fn handle_incoming(&mut self, last: bool, length: usize, data: &[u8; MASTER_PAYLOAD_MAX_SIZE]) {
        // called when receiving a message from master
        match self.in_buffer.as_mut() {
            Some(message) => message.data.extend(&data[..length]),
            None => {
                self.in_buffer = Some(Message {
                    count: data[0],
                    tag: data[1],
                    data: data[2..length].to_vec(),
                });
            }
        };
        if last {
            // when done, remove from working queue
            self.in_queue.push_back(self.in_buffer.take().unwrap());
        }
    }

    pub fn is_outgoing_ready(&mut self) -> bool {
        // called by main loop, to see if there's anything to send, will send it afterwards
        match self.out_state {
            OutMessageState::MessageReady => {
                self.out_state = OutMessageState::MessageBeingSent;
                true
            }
            _ => false,
        }
    }

    pub fn was_message_acknowledged(&mut self) -> bool {
        match self.out_state {
            OutMessageState::MessageAcknowledged => {
                self.out_state = OutMessageState::NoMessage;
                true
            }
            _ => false,
        }
    }

    pub fn get_outgoing_slice(&mut self, data_slice: &mut [u8; MASTER_PAYLOAD_MAX_SIZE]) -> Option<SliceMeta> {
        if self.out_state != OutMessageState::MessageBeingSent {
            return None;
        }
        let meta = self.out_message.as_mut()?.get_slice_master(data_slice);
        if meta.last {
            // clear the message slot
            self.out_message = None;
            // notify kernel with a flag that message is sent
            self.out_state = OutMessageState::MessageSent;
        }
        Some(meta)
    }

    pub fn ack_slice(&mut self) -> bool {
        // returns whether or not there's more to be sent
        match self.out_state {
            OutMessageState::MessageBeingSent => true,
            OutMessageState::MessageSent => {
                self.out_state = OutMessageState::MessageAcknowledged;
                false
            }
            _ => {
                warn!("received unsolicited SubkernelMessageAck");
                false
            }
        }
    }

    pub fn accept_outgoing(&mut self, message: Vec<u8>) -> Result<(), Error> {
        // service tag skipped in kernel
        self.out_message = Some(Sliceable::new(message));
        self.out_state = OutMessageState::MessageReady;
        Ok(())
    }

    pub fn get_incoming(&mut self) -> Option<Message> {
        self.in_queue.pop_front()
    }
}

impl<'a> Manager<'_> {
    pub fn new(control: &mut kernel::Control) -> Manager {
        Manager {
            kernels: BTreeMap::new(),
            session: Session::new(0),
            control: control,
            cache: BTreeMap::new(),
            last_finished: None,
        }
    }

    pub fn add(&mut self, id: u32, last: bool, data: &[u8], data_len: usize) -> Result<(), Error> {
        let kernel = match self.kernels.get_mut(&id) {
            Some(kernel) => {
                if kernel.complete {
                    // replace entry
                    self.kernels.remove(&id);
                    self.kernels.insert(
                        id,
                        KernelLibrary {
                            library: Vec::new(),
                            complete: false,
                        },
                    );
                    self.kernels.get_mut(&id)?
                } else {
                    kernel
                }
            }
            None => {
                self.kernels.insert(
                    id,
                    KernelLibrary {
                        library: Vec::new(),
                        complete: false,
                    },
                );
                self.kernels.get_mut(&id)?
            }
        };
        kernel.library.extend(&data[0..data_len]);

        kernel.complete = last;
        Ok(())
    }

    pub fn running(&self) -> bool {
        self.session.running()
    }

    pub fn get_current_id(&self) -> Option<u32> {
        match self.running() {
            true => Some(self.session.id),
            false => None,
        }
    }

    pub fn run(&mut self, id: u32) -> Result<(), Error> {
        info!("starting subkernel #{}", id);
        if self.session.kernel_state != KernelState::Loaded || self.session.id != id {
            self.load(id)?;
        }
        self.session.kernel_state = KernelState::Running;
        unsafe {
            csr::cri_con::selected_write(2);
        }

        self.control.tx.send(kernel::Message::StartRequest);
        Ok(())
    }

    pub fn message_handle_incoming(&mut self, last: bool, length: usize, slice: &[u8; MASTER_PAYLOAD_MAX_SIZE]) {
        if !self.running() {
            return;
        }
        self.session.messages.handle_incoming(last, length, slice);
    }

    pub fn message_get_slice(&mut self, slice: &mut [u8; MASTER_PAYLOAD_MAX_SIZE]) -> Option<SliceMeta> {
        if !self.running() {
            return None;
        }
        self.session.messages.get_outgoing_slice(slice)
    }

    pub fn message_ack_slice(&mut self) -> bool {
        if !self.running() {
            warn!("received unsolicited SubkernelMessageAck");
            return false;
        }
        self.session.messages.ack_slice()
    }

    pub fn message_is_ready(&mut self) -> bool {
        self.session.messages.is_outgoing_ready()
    }

    pub fn load(&mut self, id: u32) -> Result<(), Error> {
        if self.session.id == id && self.session.kernel_state == KernelState::Loaded {
            return Ok(());
        }
        if !self.kernels.get(&id)?.complete {
            return Err(Error::KernelNotFound);
        }
        self.session = Session::new(id);
        self.control.restart();

        self.control
            .tx
            .send(kernel::Message::LoadRequest(self.kernels.get(&id)?.library.clone()));
        let reply = self.control.rx.recv();
        match reply {
            kernel::Message::LoadCompleted => Ok(()),
            kernel::Message::LoadFailed => Err(Error::Load("kernel load failed".to_string())),
            _ => Err(Error::Load(format!(
                "unexpected kernel CPU reply to load request: {:?}",
                reply
            ))),
        }
    }

    pub fn exception_get_slice(&mut self, data_slice: &mut [u8; SAT_PAYLOAD_MAX_SIZE]) -> SliceMeta {
        match self.session.last_exception.as_mut() {
            Some(exception) => exception.get_slice_sat(data_slice),
            None => SliceMeta { len: 0, last: true },
        }
    }

    pub fn get_last_finished(&mut self) -> Option<SubkernelFinished> {
        self.last_finished.take()
    }

    fn kernel_stop(&mut self) {
        self.session.kernel_state = KernelState::Absent;
        unsafe {
            csr::cri_con::selected_write(0);
        }
    }

    fn runtime_exception(&mut self, cause: Error) {
        let raw_exception: Vec<u8> = Vec::new();
        let mut writer = Cursor::new(raw_exception);
        match write_exception(
            &mut writer,
            &[Some(eh_artiq::Exception {
                id: 11, // SubkernelError, defined in ksupport
                message: format!("in subkernel id {}: {:?}", self.session.id, cause).as_c_slice(),
                param: [0, 0, 0],
                file: file!().as_c_slice(),
                line: line!(),
                column: column!(),
                function: format!("subkernel id {}", self.session.id).as_c_slice(),
            })],
            &[eh_artiq::StackPointerBacktrace {
                stack_pointer: 0,
                initial_backtrace_size: 0,
                current_backtrace_size: 0,
            }],
            &[],
            0,
        ) {
            Ok(_) => self.session.last_exception = Some(Sliceable::new(writer.into_inner())),
            Err(_) => error!("Error writing exception data"),
        }
        self.kernel_stop();
    }

    pub fn process_kern_requests(&mut self, rank: u8, timer: GlobalTimer) {
        if !self.running() {
            return;
        }

        match self.process_external_messages(timer) {
            Ok(()) => (),
            Err(Error::AwaitingMessage) => return, // kernel still waiting, do not process kernel messages
            Err(Error::KernelException(exception)) => {
                self.session.last_exception = Some(exception);
                self.last_finished = Some(SubkernelFinished {
                    id: self.session.id,
                    with_exception: true,
                });
            }
            Err(e) => {
                error!("Error while running processing external messages: {:?}", e);
                self.runtime_exception(e);
                self.last_finished = Some(SubkernelFinished {
                    id: self.session.id,
                    with_exception: true,
                });
            }
        }

        match self.process_kern_message(rank, timer) {
            Ok(true) => {
                self.last_finished = Some(SubkernelFinished {
                    id: self.session.id,
                    with_exception: false,
                });
            }
            Ok(false) | Err(Error::NoMessage) => (),
            Err(Error::KernelException(exception)) => {
                self.session.last_exception = Some(exception);
                self.last_finished = Some(SubkernelFinished {
                    id: self.session.id,
                    with_exception: true,
                });
            }
            Err(e) => {
                error!("Error while running kernel: {:?}", e);
                self.runtime_exception(e);
                self.last_finished = Some(SubkernelFinished {
                    id: self.session.id,
                    with_exception: true,
                });
            }
        }
    }

    fn process_kern_message(&mut self, rank: u8, timer: GlobalTimer) -> Result<bool, Error> {
        let reply = self.control.rx.try_recv()?;
        match reply {
            kernel::Message::KernelFinished(_async_errors) => {
                self.kernel_stop();
                return Ok(true);
            }
            kernel::Message::KernelException(exceptions, stack_pointers, backtrace, async_errors) => {
                error!("exception in kernel");
                for exception in exceptions {
                    error!("{:?}", exception.unwrap());
                }
                error!("stack pointers: {:?}", stack_pointers);
                error!("backtrace: {:?}", backtrace);
                let buf: Vec<u8> = Vec::new();
                let mut writer = Cursor::new(buf);
                match write_exception(&mut writer, exceptions, stack_pointers, backtrace, async_errors) {
                    Ok(()) => (),
                    Err(_) => error!("Error writing exception data"),
                }
                self.kernel_stop();
                return Err(Error::KernelException(Sliceable::new(writer.into_inner())));
            }
            kernel::Message::CachePutRequest(key, value) => {
                self.cache.insert(key, value);
            }
            kernel::Message::CacheGetRequest(key) => {
                const DEFAULT: Vec<i32> = Vec::new();
                let value = self.cache.get(&key).unwrap_or(&DEFAULT).clone();
                self.control.tx.send(kernel::Message::CacheGetReply(value));
            }
            kernel::Message::SubkernelMsgSend { id: _, data } => {
                self.session.messages.accept_outgoing(data)?;
                self.session.kernel_state = KernelState::MsgSending;
            }
            kernel::Message::SubkernelMsgRecvRequest { id: _, timeout } => {
                let max_time = timer.get_time() + Milliseconds(timeout);
                self.session.kernel_state = KernelState::MsgAwait(max_time);
            }
            kernel::Message::UpDestinationsRequest(destination) => {
                self.control
                    .tx
                    .send(kernel::Message::UpDestinationsReply(destination == (rank as i32)));
            }
            _ => {
                unexpected!("unexpected message from core1 while kernel was running: {:?}", reply);
            }
        }
        Ok(false)
    }

    fn process_external_messages(&mut self, timer: GlobalTimer) -> Result<(), Error> {
        match self.session.kernel_state {
            KernelState::MsgAwait(timeout) => {
                if timer.get_time() > timeout {
                    self.control.tx.send(kernel::Message::SubkernelMsgRecvReply {
                        status: kernel::SubkernelStatus::Timeout,
                        count: 0,
                    });
                    self.session.kernel_state = KernelState::Running;
                    return Ok(());
                }
                if let Some(message) = self.session.messages.get_incoming() {
                    self.control.tx.send(kernel::Message::SubkernelMsgRecvReply {
                        status: kernel::SubkernelStatus::NoError,
                        count: message.count,
                    });
                    self.session.kernel_state = KernelState::Running;
                    self.pass_message_to_kernel(&message, timer)
                } else {
                    Err(Error::AwaitingMessage)
                }
            }
            KernelState::MsgSending => {
                if self.session.messages.was_message_acknowledged() {
                    self.session.kernel_state = KernelState::Running;
                    self.control.tx.send(kernel::Message::SubkernelMsgSent);
                    Ok(())
                } else {
                    Err(Error::AwaitingMessage)
                }
            }
            _ => Ok(()),
        }
    }

    fn pass_message_to_kernel(&mut self, message: &Message, timer: GlobalTimer) -> Result<(), Error> {
        let mut reader = Cursor::new(&message.data);
        let mut tag: [u8; 1] = [message.tag];
        let mut i = message.count;
        loop {
            let slot = match recv_w_timeout(&mut self.control.rx, timer, 100)? {
                kernel::Message::RpcRecvRequest(slot) => slot,
                other => unexpected!("expected root value slot from core1, not {:?}", other),
            };
            let mut exception: Option<Sliceable> = None;
            let mut unexpected: Option<String> = None;
            rpc::recv_return(&mut reader, &tag, slot, &mut |size| {
                if size == 0 {
                    0 as *mut ()
                } else {
                    self.control.tx.send(kernel::Message::RpcRecvReply(Ok(size)));
                    match recv_w_timeout(&mut self.control.rx, timer, 100) {
                        Ok(kernel::Message::RpcRecvRequest(slot)) => slot,
                        Ok(kernel::Message::KernelException(exceptions, stack_pointers, backtrace, async_errors)) => {
                            let buf: Vec<u8> = Vec::new();
                            let mut writer = Cursor::new(buf);
                            match write_exception(&mut writer, exceptions, stack_pointers, backtrace, async_errors) {
                                Ok(()) => {
                                    exception = Some(Sliceable::new(writer.into_inner()));
                                }
                                Err(_) => {
                                    unexpected = Some("Error writing exception data".to_string());
                                }
                            };
                            0 as *mut ()
                        }
                        other => {
                            unexpected = Some(format!("expected nested value slot from kernel CPU, not {:?}", other));
                            0 as *mut ()
                        }
                    }
                }
            })?;
            if let Some(exception) = exception {
                self.kernel_stop();
                return Err(Error::KernelException(exception));
            } else if let Some(unexpected) = unexpected {
                self.kernel_stop();
                unexpected!("{}", unexpected);
            }
            self.control.tx.send(kernel::Message::RpcRecvReply(Ok(0)));
            i -= 1;
            if i == 0 {
                break;
            } else {
                // update the tag for next read
                tag[0] = reader.read_u8()?;
            }
        }
        Ok(())
    }
}

fn write_exception<W>(
    writer: &mut W,
    exceptions: &[Option<eh_artiq::Exception>],
    stack_pointers: &[eh_artiq::StackPointerBacktrace],
    backtrace: &[(usize, usize)],
    async_errors: u8,
) -> Result<(), Error>
where
    W: Write + ?Sized,
{
    /* header */
    writer.write_bytes(&[0x5a, 0x5a, 0x5a, 0x5a, /*Reply::KernelException*/ 9])?;
    writer.write_u32(exceptions.len() as u32)?;
    for exception in exceptions.iter() {
        let exception = exception.as_ref().unwrap();
        writer.write_u32(exception.id)?;

        if exception.message.len() == usize::MAX {
            // exception with host string
            writer.write_u32(u32::MAX)?;
            writer.write_u32(exception.message.as_ptr() as u32)?;
        } else {
            let msg =
                str::from_utf8(unsafe { slice::from_raw_parts(exception.message.as_ptr(), exception.message.len()) })
                    .unwrap()
                    .replace(
                        "{rtio_channel_info:0}",
                        &format!(
                            "0x{:04x}:{}",
                            exception.param[0],
                            ksupport::resolve_channel_name(exception.param[0] as u32)
                        ),
                    );
            writer.write_string(&msg)?;
        }
        writer.write_u64(exception.param[0] as u64)?;
        writer.write_u64(exception.param[1] as u64)?;
        writer.write_u64(exception.param[2] as u64)?;
        writer.write_bytes(exception.file.as_ref())?;
        writer.write_u32(exception.line)?;
        writer.write_u32(exception.column)?;
        writer.write_bytes(exception.function.as_ref())?;
    }

    for sp in stack_pointers.iter() {
        writer.write_u32(sp.stack_pointer as u32)?;
        writer.write_u32(sp.initial_backtrace_size as u32)?;
        writer.write_u32(sp.current_backtrace_size as u32)?;
    }
    writer.write_u32(backtrace.len() as u32)?;
    for &(addr, sp) in backtrace {
        writer.write_u32(addr as u32)?;
        writer.write_u32(sp as u32)?;
    }
    writer.write_u8(async_errors as u8)?;
    Ok(())
}

fn recv_w_timeout(
    rx: &mut Receiver<'_, kernel::Message>,
    timer: GlobalTimer,
    timeout: u64,
) -> Result<kernel::Message, Error> {
    let max_time = timer.get_time() + Milliseconds(timeout);
    while timer.get_time() < max_time {
        match rx.try_recv() {
            Err(_) => (),
            Ok(message) => return Ok(message),
        }
    }
    Err(Error::NoMessage)
}
