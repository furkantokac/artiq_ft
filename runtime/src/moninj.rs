use core::fmt;
use alloc::collections::BTreeMap;
use log::{debug, warn};
use void::Void;

use libboard_zynq::{smoltcp, timer::GlobalTimer, time::Milliseconds};
use libasync::{task, smoltcp::TcpStream, block_async, nb};

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use futures::{pin_mut, select_biased, FutureExt};

use crate::proto::*;
use crate::pl::csr;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    NetworkError(smoltcp::Error),
    UnexpectedPattern,
    UnrecognizedPacket,

}

pub type Result<T> = core::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::NetworkError(error) => write!(f, "network error: {}", error),
            &Error::UnexpectedPattern   => write!(f, "unexpected pattern"),
            &Error::UnrecognizedPacket  => write!(f, "unrecognized packet"),
        }
    }
}

impl From<smoltcp::Error> for Error {
    fn from(error: smoltcp::Error) -> Self {
        Error::NetworkError(error)
    }
}

#[derive(Debug, FromPrimitive, ToPrimitive)]
enum HostMessage {
    MonitorProbe = 0,
    MonitorInjection = 3,
    Inject = 1,
    GetInjectionStatus = 2
}

#[derive(Debug, FromPrimitive, ToPrimitive)]
enum DeviceMessage {
    MonitorStatus = 0,
    InjectionStatus = 1
}

fn read_probe(channel: i32, probe: u8) -> i32 {
    unsafe {
        csr::rtio_moninj::mon_chan_sel_write(channel as _);
        csr::rtio_moninj::mon_probe_sel_write(probe as _);
        csr::rtio_moninj::mon_value_update_write(1);
        csr::rtio_moninj::mon_value_read() as i32
    }
}

fn inject(channel: i32, overrd: i8, value: i8) {
    unsafe {
        csr::rtio_moninj::inj_chan_sel_write(channel as _);
        csr::rtio_moninj::inj_override_sel_write(overrd as _);
        csr::rtio_moninj::inj_value_write(value as _);
    }
}

fn read_injection_status(channel: i32, overrd: i8) -> i8 {
    unsafe {
        csr::rtio_moninj::inj_chan_sel_write(channel as _);
        csr::rtio_moninj::inj_override_sel_write(overrd as _);
        csr::rtio_moninj::inj_value_read() as i8
    }
}

async fn handle_connection(stream: &TcpStream, timer: GlobalTimer) -> Result<()> {
    if !expect(&stream, b"ARTIQ moninj\n").await? {
        return Err(Error::UnexpectedPattern);
    }

    let mut probe_watch_list: BTreeMap<(i32, i8), Option<i32>> = BTreeMap::new();
    let mut inject_watch_list: BTreeMap<(i32, i8), Option<i8>> = BTreeMap::new();
    let mut next_check = Milliseconds(0);
    loop {
        // TODO: we don't need fuse() here.
        // remove after https://github.com/rust-lang/futures-rs/issues/1989 lands
        let read_message_f = read_i8(&stream).fuse();
        let next_check_c = next_check.clone();
        let timeout = || -> nb::Result<(), Void> {
            if timer.get_time() < next_check_c {
                Err(nb::Error::WouldBlock)
            } else {
                Ok(())
            }
        };
        let timeout_f = block_async!(timeout()).fuse();
        pin_mut!(read_message_f, timeout_f);
        select_biased! {
            message = read_message_f => {
                let message: HostMessage = FromPrimitive::from_i8(message?)
                    .ok_or(Error::UnrecognizedPacket)?;
                match message {
                    HostMessage::MonitorProbe => {
                        let enable = read_bool(&stream).await?;
                        let channel = read_i32(&stream).await?;
                        let probe = read_i8(&stream).await?;
                        if enable {
                            let _ = probe_watch_list.entry((channel, probe)).or_insert(None);
                            debug!("START monitoring channel {}, probe {}", channel, probe);
                        } else {
                            let _ = probe_watch_list.remove(&(channel, probe));
                            debug!("END monitoring channel {}, probe {}", channel, probe);
                        }
                    },
                    HostMessage::MonitorInjection => {
                        let enable = read_bool(&stream).await?;
                        let channel = read_i32(&stream).await?;
                        let overrd = read_i8(&stream).await?;
                        if enable {
                            let _ = inject_watch_list.entry((channel, overrd)).or_insert(None);
                            debug!("START monitoring channel {}, overrd {}", channel, overrd);
                        } else {
                            let _ = inject_watch_list.remove(&(channel, overrd));
                            debug!("END monitoring channel {}, overrd {}", channel, overrd);
                        }
                    },
                    HostMessage::Inject => {
                        let channel = read_i32(&stream).await?;
                        let overrd = read_i8(&stream).await?;
                        let value = read_i8(&stream).await?;
                        inject(channel, overrd, value);
                        debug!("INJECT channel {}, overrd {}, value {}", channel, overrd, value);
                    },
                    HostMessage::GetInjectionStatus => {
                        let channel = read_i32(&stream).await?;
                        let overrd = read_i8(&stream).await?;
                        let value = read_injection_status(channel, overrd);
                        write_i8(&stream, DeviceMessage::InjectionStatus.to_i8().unwrap()).await?;
                        write_i32(&stream, channel).await?;
                        write_i8(&stream, overrd).await?;
                        write_i8(&stream, value).await?;
                    },
                }
            },
            _ = timeout_f => {
                warn!("tick");
                next_check = next_check + Milliseconds(200);
            }
        }
    }
}

pub fn start(timer: GlobalTimer) {
    task::spawn(async move {
        loop {
            let stream = TcpStream::accept(1383, 2048, 2048).await.unwrap();
            task::spawn(async move {
                let _ = handle_connection(&stream, timer)
                    .await
                    .map_err(|e| warn!("connection terminated: {}", e));
                let _ = stream.flush().await;
                let _ = stream.abort().await;
            });
        }
    });
}
