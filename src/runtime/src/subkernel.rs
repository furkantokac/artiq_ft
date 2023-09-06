use alloc::{collections::BTreeMap, rc::Rc, vec::Vec};

use libasync::task;
use libboard_artiq::{drtio_routing::RoutingTable, drtioaux_proto::MASTER_PAYLOAD_MAX_SIZE};
use libboard_zynq::{time::Milliseconds, timer::GlobalTimer};
use libcortex_a9::mutex::Mutex;
use log::error;

use crate::rtio_mgt::drtio;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FinishStatus {
    Ok,
    CommLost,
    Exception,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SubkernelState {
    NotLoaded,
    Uploaded,
    Running,
    Finished { status: FinishStatus },
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error {
    Timeout,
    IncorrectState,
    SubkernelNotFound,
    CommLost,
    DrtioError(&'static str),
}

impl From<&'static str> for Error {
    fn from(value: &'static str) -> Error {
        Error::DrtioError(value)
    }
}

pub struct SubkernelFinished {
    pub id: u32,
    pub status: FinishStatus,
    pub exception: Option<Vec<u8>>,
}

struct Subkernel {
    pub destination: u8,
    pub data: Vec<u8>,
    pub state: SubkernelState,
}

impl Subkernel {
    pub fn new(destination: u8, data: Vec<u8>) -> Self {
        Subkernel {
            destination: destination,
            data: data,
            state: SubkernelState::NotLoaded,
        }
    }
}

static SUBKERNELS: Mutex<BTreeMap<u32, Subkernel>> = Mutex::new(BTreeMap::new());

pub async fn add_subkernel(id: u32, destination: u8, kernel: Vec<u8>) {
    SUBKERNELS
        .async_lock()
        .await
        .insert(id, Subkernel::new(destination, kernel));
}

pub async fn upload(
    aux_mutex: &Rc<Mutex<bool>>,
    routing_table: &RoutingTable,
    timer: GlobalTimer,
    id: u32,
) -> Result<(), Error> {
    if let Some(subkernel) = SUBKERNELS.async_lock().await.get_mut(&id) {
        drtio::subkernel_upload(
            aux_mutex,
            routing_table,
            timer,
            id,
            subkernel.destination,
            &subkernel.data,
        )
        .await?;
        subkernel.state = SubkernelState::Uploaded;
        Ok(())
    } else {
        Err(Error::SubkernelNotFound)
    }
}

pub async fn load(
    aux_mutex: &Rc<Mutex<bool>>,
    routing_table: &RoutingTable,
    timer: GlobalTimer,
    id: u32,
    run: bool,
) -> Result<(), Error> {
    if let Some(subkernel) = SUBKERNELS.async_lock().await.get_mut(&id) {
        if subkernel.state != SubkernelState::Uploaded {
            return Err(Error::IncorrectState);
        }
        drtio::subkernel_load(aux_mutex, routing_table, timer, id, subkernel.destination, run).await?;
        if run {
            subkernel.state = SubkernelState::Running;
        }
        Ok(())
    } else {
        Err(Error::SubkernelNotFound)
    }
}

pub async fn clear_subkernels() {
    SUBKERNELS.async_lock().await.clear();
    MESSAGE_QUEUE.async_lock().await.clear();
    CURRENT_MESSAGES.async_lock().await.clear();
}

pub async fn subkernel_finished(id: u32, with_exception: bool) {
    // called upon receiving DRTIO SubkernelRunDone
    // may be None if session ends and is cleared
    if let Some(subkernel) = SUBKERNELS.async_lock().await.get_mut(&id) {
        subkernel.state = SubkernelState::Finished {
            status: match with_exception {
                true => FinishStatus::Exception,
                false => FinishStatus::Ok,
            },
        }
    }
}

pub async fn destination_changed(
    aux_mutex: &Rc<Mutex<bool>>,
    routing_table: &RoutingTable,
    timer: GlobalTimer,
    destination: u8,
    up: bool,
) {
    let mut locked_subkernels = SUBKERNELS.async_lock().await;
    for (id, subkernel) in locked_subkernels.iter_mut() {
        if subkernel.destination == destination {
            if up {
                match drtio::subkernel_upload(aux_mutex, routing_table, timer, *id, destination, &subkernel.data).await
                {
                    Ok(_) => subkernel.state = SubkernelState::Uploaded,
                    Err(e) => error!("Error adding subkernel on destination {}: {}", destination, e),
                }
            } else {
                subkernel.state = match subkernel.state {
                    SubkernelState::Running => SubkernelState::Finished {
                        status: FinishStatus::CommLost,
                    },
                    _ => SubkernelState::NotLoaded,
                }
            }
        }
    }
}

pub async fn await_finish(
    aux_mutex: &Rc<Mutex<bool>>,
    routing_table: &RoutingTable,
    timer: GlobalTimer,
    id: u32,
    timeout: u64,
) -> Result<SubkernelFinished, Error> {
    match SUBKERNELS.async_lock().await.get(&id).unwrap().state {
        SubkernelState::Running | SubkernelState::Finished { .. } => (),
        _ => return Err(Error::IncorrectState),
    }
    let max_time = timer.get_time() + Milliseconds(timeout);
    while timer.get_time() < max_time {
        {
            match SUBKERNELS.async_lock().await.get(&id).unwrap().state {
                SubkernelState::Finished { .. } => break,
                _ => (),
            };
        }
        task::r#yield().await;
    }
    if timer.get_time() >= max_time {
        error!("Remote subkernel finish await timed out");
        return Err(Error::Timeout);
    }
    if let Some(subkernel) = SUBKERNELS.async_lock().await.get_mut(&id) {
        match subkernel.state {
            SubkernelState::Finished { status } => {
                subkernel.state = SubkernelState::Uploaded;
                Ok(SubkernelFinished {
                    id: id,
                    status: status,
                    exception: if status == FinishStatus::Exception {
                        Some(
                            drtio::subkernel_retrieve_exception(aux_mutex, routing_table, timer, subkernel.destination)
                                .await?,
                        )
                    } else {
                        None
                    },
                })
            }
            _ => Err(Error::IncorrectState),
        }
    } else {
        Err(Error::SubkernelNotFound)
    }
}

struct Message {
    from_id: u32,
    pub tag: u8,
    pub data: Vec<u8>,
}

// FIFO queue of messages
static MESSAGE_QUEUE: Mutex<Vec<Message>> = Mutex::new(Vec::new());
// currently under construction message(s) (can be from multiple sources)
static CURRENT_MESSAGES: Mutex<BTreeMap<u32, Message>> = Mutex::new(BTreeMap::new());

pub async fn message_handle_incoming(id: u32, last: bool, length: usize, data: &[u8; MASTER_PAYLOAD_MAX_SIZE]) {
    // called when receiving a message from satellite
    if SUBKERNELS.async_lock().await.get(&id).is_none() {
        // do not add messages for non-existing or deleted subkernels
        return;
    }
    let mut current_messages = CURRENT_MESSAGES.async_lock().await;
    match current_messages.get_mut(&id) {
        Some(message) => message.data.extend(&data[..length]),
        None => {
            current_messages.insert(
                id,
                Message {
                    from_id: id,
                    tag: data[0],
                    data: data[1..length].to_vec(),
                },
            );
        }
    };
    if last {
        // when done, remove from working queue
        MESSAGE_QUEUE
            .async_lock()
            .await
            .push(current_messages.remove(&id).unwrap());
    }
}

pub async fn message_await(id: u32, timeout: u64, timer: GlobalTimer) -> Result<(u8, Vec<u8>), Error> {
    match SUBKERNELS.async_lock().await.get(&id).unwrap().state {
        SubkernelState::Finished {
            status: FinishStatus::CommLost,
        } => return Err(Error::CommLost),
        SubkernelState::Running | SubkernelState::Finished { .. } => (),
        _ => return Err(Error::IncorrectState),
    }
    let max_time = timer.get_time() + Milliseconds(timeout);
    while timer.get_time() < max_time {
        {
            let mut message_queue = MESSAGE_QUEUE.async_lock().await;
            for i in 0..message_queue.len() {
                let msg = &message_queue[i];
                if msg.from_id == id {
                    let message = message_queue.remove(i);
                    return Ok((message.tag, message.data));
                }
            }
        }
        task::r#yield().await;
    }
    Err(Error::Timeout)
}

pub async fn message_send<'a>(
    aux_mutex: &Rc<Mutex<bool>>,
    routing_table: &RoutingTable,
    timer: GlobalTimer,
    id: u32,
    message: Vec<u8>,
) -> Result<(), Error> {
    let destination = SUBKERNELS.async_lock().await.get(&id).unwrap().destination;
    // rpc data prepared by the kernel core already
    Ok(drtio::subkernel_send_message(aux_mutex, routing_table, timer, id, destination, &message).await?)
}
