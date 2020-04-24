use core::fmt;
use log::{info, warn};

use libboard_zynq::smoltcp;
use libasync::task;
use libasync::smoltcp::TcpStream;

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

use crate::proto::*;


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

async fn handle_connection(stream: &TcpStream) -> Result<()> {
    if !expect(&stream, b"ARTIQ moninj\n").await? {
        return Err(Error::UnexpectedPattern);
    }
    loop {
        let message: HostMessage = FromPrimitive::from_i8(read_i8(&stream).await?)
            .ok_or(Error::UnrecognizedPacket)?;
        info!("{:?}", message);
        match message {
            HostMessage::MonitorProbe => {
                let enable = read_bool(&stream).await?;
                let channel = read_i32(&stream).await?;
                let probe = read_i8(&stream).await?;
            },
            HostMessage::Inject => {
                let channel = read_i32(&stream).await?;
                let overrd = read_i8(&stream).await?;
                let value = read_i8(&stream).await?;
            },
            HostMessage::GetInjectionStatus => {
                let channel = read_i32(&stream).await?;
                let overrd = read_i8(&stream).await?;
            },
            HostMessage::MonitorInjection => {
                let enable = read_bool(&stream).await?;
                let channel = read_i32(&stream).await?;
                let overrd = read_i8(&stream).await?;
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
