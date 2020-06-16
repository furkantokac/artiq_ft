use crate::sd_reader;
use core_io::{Error, Read, Seek, SeekFrom};
use libboard_zynq::{devc, sdio};
use log::{info, debug};

#[derive(Debug)]
pub enum PlLoadingError {
    BootImageNotFound,
    InvalidBootImageHeader,
    MissingBitstreamPartition,
    EncryptedBitstream,
    IoError(Error),
    DevcError(devc::DevcError),
}

impl From<Error> for PlLoadingError {
    fn from(error: Error) -> Self {
        PlLoadingError::IoError(error)
    }
}

impl From<devc::DevcError> for PlLoadingError {
    fn from(error: devc::DevcError) -> Self {
        PlLoadingError::DevcError(error)
    }
}

impl core::fmt::Display for PlLoadingError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use PlLoadingError::*;
        match self {
            BootImageNotFound => write!(
                f,
                "Boot image not found, make sure `boot.bin` exists and your SD card is plugged in."
            ),
            InvalidBootImageHeader => write!(
                f,
                "Invalid boot image header. Check if the file is correct."
            ),
            MissingBitstreamPartition => write!(
                f,
                "Bitstream partition not found. Check your compile configuration."
            ),
            EncryptedBitstream => write!(f, "Encrypted bitstream is not supported."),
            IoError(e) => write!(f, "Error while reading: {}", e),
            DevcError(e) => write!(f, "PCAP interface error: {}", e),
        }
    }
}

#[repr(C)]
struct PartitionHeader {
    pub encrypted_length: u32,
    pub unencrypted_length: u32,
    pub word_length: u32,
    pub dest_load_addr: u32,
    pub dest_exec_addr: u32,
    pub data_offset: u32,
    pub attribute_bits: u32,
    pub section_count: u32,
    pub checksum_offset: u32,
    pub header_offset: u32,
    pub cert_offset: u32,
    pub reserved: [u32; 4],
    pub checksum: u32,
}

/// Read a u32 word from the reader.
fn read_u32<Reader: Read>(reader: &mut Reader) -> Result<u32, PlLoadingError> {
    let mut buffer: [u8; 4] = [0; 4];
    reader.read_exact(&mut buffer)?;
    let mut result: u32 = 0;
    for i in 0..4 {
        result |= (buffer[i] as u32) << (i * 8);
    }
    Ok(result)
}

/// Load PL partition header.
fn load_pl_header<File: Read + Seek>(
    file: &mut File,
) -> Result<Option<PartitionHeader>, PlLoadingError> {
    let mut buffer: [u8; 0x40] = [0; 0x40];
    file.read_exact(&mut buffer)?;
    let header = unsafe { core::mem::transmute::<_, PartitionHeader>(buffer) };
    if header.attribute_bits & (2 << 4) != 0 {
        Ok(Some(header))
    } else {
        Ok(None)
    }
}

/// Locate the PL bitstream from the image, and return the size (in bytes) of the bitstream if successful.
/// This function would seek the file to the location of the bitstream.
fn locate_bitstream<File: Read + Seek>(file: &mut File) -> Result<usize, PlLoadingError> {
    const BOOT_HEADER_SIGN: u32 = 0x584C4E58;
    // read boot header signature
    file.seek(SeekFrom::Start(0x24))?;
    if read_u32(file)? != BOOT_HEADER_SIGN {
        return Err(PlLoadingError::InvalidBootImageHeader);
    }
    // read partition header offset
    file.seek(SeekFrom::Start(0x9C))?;
    let ptr = read_u32(file)?;
    debug!("Partition header pointer = {:0X}", ptr);
    file.seek(SeekFrom::Start(ptr as u64))?;

    let mut header_opt = None;
    // at most 3 partition headers
    for _ in 0..3 {
        let result = load_pl_header(file)?;
        if let Some(h) = result {
            header_opt = Some(h);
            break;
        }
    }
    let header = match header_opt {
        None => return Err(PlLoadingError::MissingBitstreamPartition),
        Some(h) => h,
    };

    let encrypted_length = header.encrypted_length;
    let unencrypted_length = header.unencrypted_length;
    debug!("Unencrypted length = {:0X}", unencrypted_length);
    if encrypted_length != unencrypted_length {
        return Err(PlLoadingError::EncryptedBitstream);
    }

    let start_addr = header.data_offset;
    debug!("Partition start address: {:0X}", start_addr);
    file.seek(SeekFrom::Start(start_addr as u64 * 4))?;

    Ok(unencrypted_length as usize * 4)
}

/// Load bitstream from bootgen file.
/// This function parses the file, locate the bitstream and load it through the PCAP driver.
/// It requires a large buffer, please enable the DDR RAM before using it.
pub fn load_bitstream<File: Read + Seek>(
    file: &mut File,
) -> Result<(), PlLoadingError> {
    let size = locate_bitstream(file)?;
    let mut buffer: alloc::vec::Vec<u8> = alloc::vec::Vec::with_capacity(size);
    unsafe {
        buffer.set_len(buffer.capacity());
    }
    file.read_exact(&mut buffer)?;

    let mut devcfg = devc::DevC::new();
    devcfg.enable();
    devcfg.program(&buffer)?;
    Ok(())
}

pub fn load_bitstream_from_sd() -> Result<(), PlLoadingError> {
    let sdio0 = sdio::SDIO::sdio0(true);
    if sdio0.is_card_inserted() {
        info!("Card inserted. Mounting file system.");
        let mut sd = sdio::sd_card::SdCard::from_sdio(sdio0).unwrap();
        let reader = sd_reader::SdReader::new(&mut sd);

        let fs = reader.mount_fatfs(sd_reader::PartitionEntry::Entry1)?;
        let root_dir = fs.root_dir();
        for entry in root_dir.iter() {
            if let Ok(entry) = entry {
                if entry.is_file() && entry.short_file_name() == "BOOT.BIN" {
                    info!("Found boot image!");
                    return load_bitstream(&mut entry.to_file());
                }
            }
        }
    } else {
        info!("SD card not inserted. Bitstream cannot be loaded.")
    }
    Err(PlLoadingError::BootImageNotFound)
}
