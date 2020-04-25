use core::fmt;
use alloc::collections::BTreeMap;
use log::warn;

use libboard_zynq::smoltcp;
use libasync::task;
use libasync::smoltcp::TcpStream;

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

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

async fn handle_connection(stream: &TcpStream) -> Result<()> {
    if !expect(&stream, b"ARTIQ moninj\n").await? {
        return Err(Error::UnexpectedPattern);
    }

    let mut probe_watch_list: BTreeMap<(i32, i8), Option<i32>> = BTreeMap::new();
    let mut inject_watch_list: BTreeMap<(i32, i8), Option<i8>> = BTreeMap::new();

    loop {
        let message: HostMessage = FromPrimitive::from_i8(read_i8(&stream).await?)
            .ok_or(Error::UnrecognizedPacket)?;
        match message {
            HostMessage::MonitorProbe => {
                let enable = read_bool(&stream).await?;
                let channel = read_i32(&stream).await?;
                let probe = read_i8(&stream).await?;
                if enable {
                    let _ = probe_watch_list.entry((channel, probe)).or_insert(None);
                } else {
                    let _ = probe_watch_list.remove(&(channel, probe));
                }
            },
            HostMessage::MonitorInjection => {
                let enable = read_bool(&stream).await?;
                let channel = read_i32(&stream).await?;
                let overrd = read_i8(&stream).await?;
                if enable {
                    let _ = inject_watch_list.entry((channel, overrd)).or_insert(None);
                } else {
                    let _ = inject_watch_list.remove(&(channel, overrd));
                }
            },
            HostMessage::Inject => {
                let channel = read_i32(&stream).await?;
                let overrd = read_i8(&stream).await?;
                let value = read_i8(&stream).await?;
                inject(channel, overrd, value);
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
    }
}

pub fn start() {
    task::spawn(async move {
        loop {
            let stream = TcpStream::accept(1383, 2048, 2048).await.unwrap();
            task::spawn(async {
                let _ = handle_connection(&stream)
                    .await
                    .map_err(|e| warn!("connection terminated: {}", e));
                let _ = stream.flush().await;
                let _ = stream.abort().await;
            });
        }
    });
}
