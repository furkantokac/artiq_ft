use crate::sd_reader;
use core::fmt;
use alloc::{string::FromUtf8Error, string::String, vec::Vec};
use core_io::{self as io, BufRead, BufReader, Read};

use libboard_zynq::sdio;

#[derive(Debug)]
pub enum Error<'a> {
    SdError(sdio::sd_card::CardInitializationError),
    IoError(io::Error),
    Utf8Error(FromUtf8Error),
    KeyNotFoundError(&'a str),
}

pub type Result<'a, T> = core::result::Result<T, Error<'a>>;

impl<'a> fmt::Display for Error<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::SdError(error) => write!(f, "SD error: {}", error),
            Error::IoError(error) => write!(f, "I/O error: {}", error),
            Error::Utf8Error(error) => write!(f, "UTF-8 error: {}", error),
            Error::KeyNotFoundError(name) => write!(f, "Configuration key `{}` not found", name),
        }
    }
}

impl<'a> From<sdio::sd_card::CardInitializationError> for Error<'a> {
    fn from(error: sdio::sd_card::CardInitializationError) -> Self {
        Error::SdError(error)
    }
}

impl<'a> From<io::Error> for Error<'a> {
    fn from(error: io::Error) -> Self {
        Error::IoError(error)
    }
}

impl<'a> From<FromUtf8Error> for Error<'a> {
    fn from(error: FromUtf8Error) -> Self {
        Error::Utf8Error(error)
    }
}

fn parse_config<'a>(
    key: &'a str,
    buffer: &mut Vec<u8>,
    file: fatfs::File<sd_reader::SdReader>,
) -> Result<'a, ()> {
    let prefix = [key, "="].concat();
    for line in BufReader::new(file).lines() {
        let line = line?;
        if line.starts_with(&prefix) {
            buffer.extend(line[prefix.len()..].as_bytes());
            return Ok(());
        }
    }
    Err(Error::KeyNotFoundError(key))
}

pub struct Config {
    fs: fatfs::FileSystem<sd_reader::SdReader>,
}

impl Config {
    pub fn new() -> Result<'static, Self> {
        let sdio = sdio::SDIO::sdio0(true);
        if !sdio.is_card_inserted() {
            Err(sdio::sd_card::CardInitializationError::NoCardInserted)?;
        }
        let sd = sdio::sd_card::SdCard::from_sdio(sdio)?;
        let reader = sd_reader::SdReader::new(sd);

        let fs = reader.mount_fatfs(sd_reader::PartitionEntry::Entry1)?;
        Ok(Config { fs })
    }

    fn read<'b>(&mut self, key: &'b str) -> Result<'b, Vec<u8>> {
        let root_dir = self.fs.root_dir();
        let mut buffer: Vec<u8> = Vec::new();
        match root_dir.open_file(&["/CONFIG/", key, ".BIN"].concat()) {
            Ok(mut f) => f.read_to_end(&mut buffer).map(|v| ())?,
            Err(_) => match root_dir.open_file("/CONFIG.TXT") {
                Ok(f) => parse_config(key, &mut buffer, f)?,
                Err(_) => return Err(Error::KeyNotFoundError(key)),
            },
        };
        Ok(buffer)
    }

    pub fn read_str<'b>(&mut self, key: &'b str) -> Result<'b, String> {
        Ok(String::from_utf8(self.read(key)?)?)
    }
}
