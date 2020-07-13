use libasync::smoltcp::TcpStream;
use libboard_zynq::smoltcp::Error as IoError;
use log;

use crate::proto_async::*;

pub enum Error {
    WrongMagic,
    UnknownPacket(u8),
    UnknownLogLevel(u8),
    Io(IoError),
}

impl core::fmt::Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use Error::*;
        match self {
            WrongMagic => write!(f, "Wrong magic string"),
            UnknownPacket(v) => write!(f, "Unknown packet {:#02x}", v),
            UnknownLogLevel(v) => write!(f, "Unknown log level {}", v),
            Io(e) => write!(f, "{}", e),
        }
    }
}

impl From<IoError> for Error {
    fn from(value: IoError) -> Error {
        Error::Io(value)
    }
}

#[derive(Debug)]
pub enum Request {
    GetLog,
    ClearLog,
    PullLog,
    SetLogFilter(log::LevelFilter),
    SetUartLogFilter(log::LevelFilter),
}

pub enum Reply<'a> {
    Success,
    LogContent(&'a str),
}

impl Request {
    pub async fn read_from(stream: &mut TcpStream) -> Result<Self, Error> {
        async fn read_log_level_filter(stream: &mut TcpStream) -> Result<log::LevelFilter, Error> {
            Ok(match read_i8(stream).await? {
                0 => log::LevelFilter::Off,
                1 => log::LevelFilter::Error,
                2 => log::LevelFilter::Warn,
                3 => log::LevelFilter::Info,
                4 => log::LevelFilter::Debug,
                5 => log::LevelFilter::Trace,
                lv => return Err(Error::UnknownLogLevel(lv as u8)),
            })
        }

        Ok(match read_i8(stream).await? {
            1 => Request::GetLog,
            2 => Request::ClearLog,
            7 => Request::PullLog,
            3 => Request::SetLogFilter(read_log_level_filter(stream).await?),
            6 => Request::SetUartLogFilter(read_log_level_filter(stream).await?),
            ty => return Err(Error::UnknownPacket(ty as u8)),
        })
    }

    pub async fn read_magic(stream: &mut TcpStream) -> Result<(), Error> {
        if !expect(&stream, b"ARTIQ management\n").await? {
            return Err(Error::WrongMagic);
        }
        Ok(())
    }
}

impl<'a> Reply<'a> {
    pub async fn write_to(&self, stream: &mut TcpStream) -> Result<(), IoError> {
        match *self {
            Reply::Success => {
                write_i8(stream, 1).await?;
            }
            Reply::LogContent(ref log) => {
                write_i8(stream, 2).await?;
                write_chunk(stream, log.as_bytes()).await?;
            }
        }
        Ok(())
    }
}
