use log::info;
use core::fmt;
use alloc::{vec::Vec, string::String, string::FromUtf8Error};
use core_io as io;

use libboard_zynq::sdio;

use crate::sd_reader;

#[derive(Debug)]
pub enum Error {
    SdError(sdio::sd_card::CardInitializationError),
    IoError(io::Error),
    Utf8Error(FromUtf8Error),
}

pub type Result<T> = core::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::SdError(error)   => write!(f, "SD error: {:?}", error), // TODO: Display for CardInitializationError?
            Error::IoError(error)   => write!(f, "I/O error: {}", error),
            Error::Utf8Error(error) => write!(f, "UTF-8 error: {}", error),
        }
    }
}

impl From<sdio::sd_card::CardInitializationError> for Error {
    fn from(error: sdio::sd_card::CardInitializationError) -> Self {
        Error::SdError(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IoError(error)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(error: FromUtf8Error) -> Self {
        Error::Utf8Error(error)
    }
}

pub fn read(key: &str) -> Result<Vec<u8>> {
    let sdio = sdio::SDIO::sdio0(true);
    let mut sd = sdio::sd_card::SdCard::from_sdio(sdio)?;
    let reader = sd_reader::SdReader::new(&mut sd);

    let fs = reader.mount_fatfs(sd_reader::PartitionEntry::Entry1)?;
    let root_dir = fs.root_dir();
    for entry in root_dir.iter() {
        if let Ok(entry) = entry {
            let bytes = entry.short_file_name_as_bytes();
            info!("{}", core::str::from_utf8(bytes).unwrap());
        }
    }
    Err(sdio::sd_card::CardInitializationError::NoCardInserted)? // TODO
}

pub fn read_str(key: &str) -> Result<String> {
    Ok(String::from_utf8(read(key)?)?)
}
