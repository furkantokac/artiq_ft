use core_io::{BufRead, Error, ErrorKind, Read, Result as IoResult, Seek, SeekFrom, Write};
use fatfs;
use libboard_zynq::sdio::{sd_card::SdCard, CmdTransferError};
use log::debug;
use alloc::vec::Vec;

const MBR_SIGNATURE: [u8; 2] = [0x55, 0xAA];
const PARTID_FAT12: u8 = 0x01;
const PARTID_FAT16_LESS32M: u8 = 0x04;
const PARTID_FAT16: u8 = 0x06;
const PARTID_FAT32: u8 = 0x0B;
const PARTID_FAT32_LBA: u8 = 0x0C;

fn cmd_error_to_io_error(_: CmdTransferError) -> Error {
    Error::new(ErrorKind::Other, "Command transfer error")
}

const BLOCK_SIZE: usize = 512;

/// SdReader struct implementing `Read + BufRead + Write + Seek` traits for `core_io`.
/// Used as an adaptor for fatfs crate, but could be used directly for raw data access.
///
/// Implementation: all read/writes would be split into unaligned and block-aligned parts,
/// unaligned read/writes would do a buffered read/write using a block-sized internal buffer,
/// while aligned transactions would be sent to the SD card directly for performance reason.
pub struct SdReader {
    /// Internal SdCard handle.
    sd: SdCard,
    /// Read buffer with the size of 1 block.
    buffer: Vec<u8>,
    /// Address for the next byte.
    byte_addr: u32,
    /// Internal index for the next byte.
    /// Normally in range `[0, BLOCK_SIZE - 1]`.
    ///
    /// `index = BLOCK_SIZE` means that the `buffer` is invalid for the current `byte_addr`,
    /// the next `fill_buf` call would fill the buffer.
    index: usize,
    /// Dirty flag indicating the content has to be flushed.
    dirty: bool,
    /// Base offset for translation from logical address to physical address.
    offset: u32,
}

#[derive(Copy, Clone)]
#[allow(unused)]
// Partition entry enum, normally we would use entry1.
pub enum PartitionEntry {
    Entry1 = 0x1BE,
    Entry2 = 0x1CE,
    Entry3 = 0x1DE,
    Entry4 = 0x1EE,
}

impl SdReader {
    /// Create SdReader from SdCard
    pub fn new(sd: SdCard) -> SdReader {
        let mut vec: Vec<u8> = Vec::with_capacity(BLOCK_SIZE);
        unsafe {
            vec.set_len(vec.capacity());
        }
        SdReader {
            sd,
            buffer: vec,
            byte_addr: 0,
            index: BLOCK_SIZE,
            dirty: false,
            offset: 0,
        }
    }

    /// Internal read function for unaligned read.
    /// The read must not cross block boundary.
    fn read_unaligned(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        if buf.len() == 0 {
            return Ok(0);
        }
        let filled_buffer = self.fill_buf()?;
        for (dest, src) in buf.iter_mut().zip(filled_buffer.iter()) {
            *dest = *src;
        }
        self.consume(buf.len());
        Ok(buf.len())
    }

    /// Internal write function for unaligned write.
    /// The write must not cross block boundary.
    fn write_unaligned(&mut self, buf: &[u8]) -> IoResult<usize> {
        if buf.len() == 0 {
            return Ok(0);
        }
        // update buffer if needed, as we will flush the entire block later.
        self.fill_buf()?;
        self.dirty = true;
        let dest_buffer = &mut self.buffer[self.index..];
        for (src, dest) in buf.iter().zip(dest_buffer.iter_mut()) {
            *dest = *src;
        }
        self.consume(buf.len());
        Ok(buf.len())
    }

    /// Split the slice into three segments, with the middle block-aligned.
    /// Alignment depends on the current `self.byte_addr` instead of the slice pointer address
    fn block_align<'b>(&self, buf: &'b [u8]) -> (&'b [u8], &'b [u8], &'b [u8]) {
        let head_len = BLOCK_SIZE - (self.byte_addr as usize % BLOCK_SIZE);
        if head_len > buf.len() {
            (buf, &[], &[])
        } else {
            let remaining_length = buf.len() - head_len;
            let mid_length = remaining_length - remaining_length % BLOCK_SIZE;
            let (head, remaining) = buf.split_at(head_len);
            let (mid, tail) = remaining.split_at(mid_length);
            (head, mid, tail)
        }
    }

    /// Split the mutable slice into three segments, with the middle block-aligned.
    /// Alignment depends on the current `self.byte_addr` instead of the slice pointer address
    fn block_align_mut<'b>(&self, buf: &'b mut [u8]) -> (&'b mut [u8], &'b mut [u8], &'b mut [u8]) {
        let head_len = BLOCK_SIZE - (self.byte_addr as usize % BLOCK_SIZE);
        if head_len > buf.len() {
            (buf, &mut [], &mut [])
        } else {
            let remaining_length = buf.len() - head_len;
            let mid_length = remaining_length - remaining_length % BLOCK_SIZE;
            let (head, remaining) = buf.split_at_mut(head_len);
            let (mid, tail) = remaining.split_at_mut(mid_length);
            (head, mid, tail)
        }
    }

    /// Invalidate the buffer, so later unaligned read/write would reload the buffer from SD card.
    fn invalidate_buffer(&mut self) {
        self.index = BLOCK_SIZE;
    }

    /// Set the base offset of the SD card, to transform from physical address to logical address.
    fn set_base_offset(&mut self, offset: u32) -> IoResult<u64> {
        self.offset = offset;
        self.seek(SeekFrom::Start(0))
    }

    /// Mount fatfs from partition entry, and return the fatfs object if success.
    /// This takes the ownership of self, so currently there is no way to recover from an error,
    /// except creating a new SD card instance.
    pub fn mount_fatfs(mut self, entry: PartitionEntry) -> IoResult<fatfs::FileSystem<Self>> {
        let mut buffer: [u8; 4] = [0; 4];
        self.seek(SeekFrom::Start(0x1FE))?;
        self.read_exact(&mut buffer[..2])?;
        // check MBR signature
        if buffer[..2] != MBR_SIGNATURE {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Incorrect signature for MBR sector.",
            ));
        }
        // Read partition ID.
        self.seek(SeekFrom::Start(entry as u64 + 0x4))?;
        self.read_exact(&mut buffer[..1])?;
        debug!("Partition ID: {:0X}", buffer[0]);
        match buffer[0] {
            PARTID_FAT12 | PARTID_FAT16_LESS32M | PARTID_FAT16 |
            PARTID_FAT32 | PARTID_FAT32_LBA => {}
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "No FAT partition found for the specified entry.",
                ));
            }
        }
        // Read LBA
        self.seek(SeekFrom::Current(0x3))?;
        self.read_exact(&mut buffer)?;
        let mut lba: u32 = 0;
        // Little endian
        for i in 0..4 {
            lba |= (buffer[i] as u32) << (i * 8);
        }
        // Set to logical address
        self.set_base_offset(lba * BLOCK_SIZE as u32)?;
        // setup fatfs
        fatfs::FileSystem::new(self, fatfs::FsOptions::new())
    }
}

impl Read for SdReader {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        let total_length = buf.len();
        let (a, b, c) = self.block_align_mut(buf);
        self.read_unaligned(a)?;
        if b.len() > 0 {
            // invalidate internal buffer
            self.invalidate_buffer();
            if let Err(_) = self.sd.read_block(
                self.byte_addr / BLOCK_SIZE as u32,
                (b.len() / BLOCK_SIZE) as u16,
                b,
            ) {
                // we have to allow partial read, as per the trait required
                return Ok(a.len());
            }
            self.byte_addr += b.len() as u32;
        }
        if let Err(_) = self.read_unaligned(c) {
            // we have to allow partial read, as per the trait required
            return Ok(a.len() + b.len());
        }
        Ok(total_length)
    }
}

impl BufRead for SdReader {
    fn fill_buf(&mut self) -> IoResult<&[u8]> {
        if self.index == BLOCK_SIZE {
            // flush the buffer if it is dirty before overwriting it with new data
            if self.dirty {
                self.flush()?;
            }
            // reload buffer
            self.sd
                .read_block(self.byte_addr / (BLOCK_SIZE as u32), 1, &mut self.buffer)
                .map_err(cmd_error_to_io_error)?;
            self.index = (self.byte_addr as usize) % BLOCK_SIZE;
        }
        Ok(&self.buffer[self.index..])
    }

    fn consume(&mut self, amt: usize) {
        self.index += amt;
        self.byte_addr += amt as u32;
    }
}

impl Write for SdReader {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        let (a, b, c) = self.block_align(buf);
        self.write_unaligned(a)?;
        if b.len() > 0 {
            self.flush()?;
            self.invalidate_buffer();
            if let Err(_) = self.sd.write_block(
                self.byte_addr / BLOCK_SIZE as u32,
                (b.len() / BLOCK_SIZE) as u16,
                b,
            ) {
                return Ok(a.len());
            }
            self.byte_addr += b.len() as u32;
        }
        if let Err(_) = self.write_unaligned(c) {
            return Ok(a.len() + b.len());
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> IoResult<()> {
        if self.dirty {
            let block_addr = (self.byte_addr - self.index as u32) / (BLOCK_SIZE as u32);
            self.sd
                .write_block(block_addr, 1, &self.buffer)
                .map_err(cmd_error_to_io_error)?;
            self.dirty = false;
        }
        Ok(())
    }
}

impl Seek for SdReader {
    fn seek(&mut self, pos: SeekFrom) -> IoResult<u64> {
        let raw_target = match pos {
            SeekFrom::Start(x) => self.offset as i64 + x as i64,
            SeekFrom::Current(x) => self.byte_addr as i64 + x,
            SeekFrom::End(_) => panic!("SD card does not support seek from end"),
        };
        if raw_target < self.offset as i64 || raw_target > core::u32::MAX as i64 {
            return Err(Error::new(ErrorKind::InvalidInput, "Invalid address"));
        }
        let target_byte_addr = raw_target as u32;
        let address_same_block =
            self.byte_addr / (BLOCK_SIZE as u32) == target_byte_addr / (BLOCK_SIZE as u32);
        // if the buffer was invalidated, we consider seek as different block
        let same_block = address_same_block && self.index != BLOCK_SIZE;
        if !same_block {
            self.flush()?;
        }
        self.byte_addr = target_byte_addr;
        self.index = if same_block {
            target_byte_addr as usize % BLOCK_SIZE
        } else {
            // invalidate the buffer as we moved to a different block
            BLOCK_SIZE
        };
        Ok((self.byte_addr - self.offset) as u64)
    }
}

impl Drop for SdReader {
    fn drop(&mut self) {
        // just try to flush it, ignore error if any
        self.flush().unwrap_or(());
    }
}
