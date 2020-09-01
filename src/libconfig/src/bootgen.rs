use alloc::vec::Vec;
use core_io::{Error, Read, Seek, SeekFrom};
use libboard_zynq::devc;
use log::debug;

#[derive(Debug)]
pub enum BootgenLoadingError {
    InvalidBootImageHeader,
    MissingPartition,
    EncryptedBitstream,
    IoError(Error),
    DevcError(devc::DevcError),
}

impl From<Error> for BootgenLoadingError {
    fn from(error: Error) -> Self {
        BootgenLoadingError::IoError(error)
    }
}

impl From<devc::DevcError> for BootgenLoadingError {
    fn from(error: devc::DevcError) -> Self {
        BootgenLoadingError::DevcError(error)
    }
}

impl core::fmt::Display for BootgenLoadingError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use BootgenLoadingError::*;
        match self {
            InvalidBootImageHeader => write!(
                f,
                "Invalid boot image header. Check if the file is correct."
            ),
            MissingPartition => write!(f, "Partition not found. Check your compile configuration."),
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
fn read_u32<Reader: Read>(reader: &mut Reader) -> Result<u32, BootgenLoadingError> {
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
) -> Result<Option<PartitionHeader>, BootgenLoadingError> {
    let mut buffer: [u8; 0x40] = [0; 0x40];
    file.read_exact(&mut buffer)?;
    let header = unsafe { core::mem::transmute::<_, PartitionHeader>(buffer) };
    if header.attribute_bits & (2 << 4) != 0 {
        Ok(Some(header))
    } else {
        Ok(None)
    }
}

fn load_ps_header<File: Read + Seek>(
    file: &mut File,
) -> Result<Option<PartitionHeader>, BootgenLoadingError> {
    let mut buffer: [u8; 0x40] = [0; 0x40];
    file.read_exact(&mut buffer)?;
    let header = unsafe { core::mem::transmute::<_, PartitionHeader>(buffer) };
    if header.attribute_bits & (1 << 4) != 0 {
        Ok(Some(header))
    } else {
        Ok(None)
    }
}

/// Locate the partition from the image, and return the size (in bytes) of the partition if successful.
/// This function would seek the file to the location of the partition.
fn locate<
    File: Read + Seek,
    F: Fn(&mut File) -> Result<Option<PartitionHeader>, BootgenLoadingError>,
>(
    file: &mut File,
    f: F,
) -> Result<usize, BootgenLoadingError> {
    file.seek(SeekFrom::Start(0))?;
    const BOOT_HEADER_SIGN: u32 = 0x584C4E58;
    // read boot header signature
    file.seek(SeekFrom::Start(0x24))?;
    if read_u32(file)? != BOOT_HEADER_SIGN {
        return Err(BootgenLoadingError::InvalidBootImageHeader);
    }
    // find fsbl offset
    file.seek(SeekFrom::Start(0x30))?;
    // the length is in bytes, we have to convert it to words to compare with the partition offset
    // later
    let fsbl = read_u32(file)? / 4;
    // read partition header offset
    file.seek(SeekFrom::Start(0x9C))?;
    let ptr = read_u32(file)?;
    debug!("Partition header pointer = {:0X}", ptr);
    file.seek(SeekFrom::Start(ptr as u64))?;

    // at most 3 partition headers
    for _ in 0..3 {
        if let Some(header) = f(file)? {
            let encrypted_length = header.encrypted_length;
            let unencrypted_length = header.unencrypted_length;
            debug!("Unencrypted length = {:0X}", unencrypted_length);
            if encrypted_length != unencrypted_length {
                return Err(BootgenLoadingError::EncryptedBitstream);
            }

            let start_addr = header.data_offset;
            // skip fsbl
            if start_addr == fsbl {
                continue;
            }
            debug!("Partition start address: {:0X}", start_addr);
            file.seek(SeekFrom::Start(start_addr as u64 * 4))?;

            return Ok(unencrypted_length as usize * 4);
        }
    }
    Err(BootgenLoadingError::MissingPartition)
}

/// Load bitstream from bootgen file.
/// This function parses the file, locate the bitstream and load it through the PCAP driver.
/// It requires a large buffer, please enable the DDR RAM before using it.
pub fn load_bitstream<File: Read + Seek>(file: &mut File) -> Result<(), BootgenLoadingError> {
    let size = locate(file, load_pl_header)?;
    unsafe {
        // align to 64 bytes
        let ptr = alloc::alloc::alloc(alloc::alloc::Layout::from_size_align(size, 64).unwrap());
        let buffer = core::slice::from_raw_parts_mut(ptr, size);
        file.read_exact(buffer).map_err(|e| {
            core::ptr::drop_in_place(ptr);
            e
        })?;
        let mut devcfg = devc::DevC::new();
        devcfg.enable();
        devcfg.program(&buffer).map_err(|e| {
            core::ptr::drop_in_place(ptr);
            e
        })?;
        core::ptr::drop_in_place(ptr);
        Ok(())
    }
}

pub fn get_runtime<File: Read + Seek>(file: &mut File) -> Result<Vec<u8>, BootgenLoadingError> {
    let size = locate(file, load_ps_header)?;
    let mut buffer = Vec::with_capacity(size);
    unsafe {
        buffer.set_len(size);
    }
    file.read_exact(&mut buffer)?;
    Ok(buffer)
}
