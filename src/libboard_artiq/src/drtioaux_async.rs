use crc;

use core_io::{ErrorKind as IoErrorKind, Error as IoError};
use void::Void;
use nb;

use libboard_zynq::{timer::GlobalTimer, time::Milliseconds};
use libasync::{task, block_async};

use io::{proto::ProtoRead, proto::ProtoWrite, Cursor};
use crate::mem::mem::DRTIOAUX_MEM;
use crate::pl::csr::DRTIOAUX;
use crate::drtioaux::{Error, has_rx_error, copy_work_buffer};

pub use crate::drtioaux_proto::Packet;

pub async fn reset(linkno: u8) {
    let linkno = linkno as usize;
    unsafe {
        // clear buffer first to limit race window with buffer overflow
        // error. We assume the CPU is fast enough so that no two packets
        // will be received between the buffer and the error flag are cleared.
        (DRTIOAUX[linkno].aux_rx_present_write)(1);
        (DRTIOAUX[linkno].aux_rx_error_write)(1);
    }
}

fn tx_ready(linkno: usize) -> nb::Result<(), Void> {
    unsafe {
        if (DRTIOAUX[linkno].aux_tx_read)() != 0 {
            Err(nb::Error::WouldBlock)
        }
        else {
            Ok(())
        }
    }
}

async fn receive<F, T>(linkno: u8, f: F) -> Result<Option<T>, Error>
    where F: FnOnce(&[u8]) -> Result<T, Error>
{
    let linkidx = linkno as usize;
    unsafe {
        if (DRTIOAUX[linkidx].aux_rx_present_read)() == 1 {
            let ptr = (DRTIOAUX_MEM[linkidx].base + DRTIOAUX_MEM[linkidx].size / 2) as *mut u32;
            let len = (DRTIOAUX[linkidx].aux_rx_length_read)() as usize;
            // work buffer to accomodate axi burst reads
            let mut buf: [u8; 1024] = [0; 1024];
            copy_work_buffer(ptr, buf.as_mut_ptr() as *mut u32, len as isize);
            let result = f(&buf[0..len]);
            (DRTIOAUX[linkidx].aux_rx_present_write)(1);
            Ok(Some(result?))
        } else {
            Ok(None)
        }
    }
}

pub async fn recv(linkno: u8) -> Result<Option<Packet>, Error> {
    if has_rx_error(linkno) {
        return Err(Error::GatewareError)
    }

    receive(linkno, |buffer| {
        if buffer.len() < 8 {
            return Err(IoError::new(IoErrorKind::UnexpectedEof, "Unexpected end").into())
        }

        let mut reader = Cursor::new(buffer);

        let checksum_at = buffer.len() - 4;
        let checksum = crc::crc32::checksum_ieee(&reader.get_ref()[0..checksum_at]);
        reader.set_position(checksum_at);
        if reader.read_u32()? != checksum {
            return Err(Error::CorruptedPacket)
        }
        reader.set_position(0);

        Ok(Packet::read_from(&mut reader)?)
    }).await
}

pub async fn recv_timeout(linkno: u8, timeout_ms: Option<u64>,
    timer: GlobalTimer) -> Result<Packet, Error> 
{
    let timeout_ms = Milliseconds(timeout_ms.unwrap_or(10));
    let limit = timer.get_time() + timeout_ms;
    let mut would_block = false;
    while timer.get_time() < limit {
        // to ensure one last time recv would run one last time
        // in case async would return after timeout
        if would_block {
            task::r#yield().await;
        }
        match recv(linkno).await? {
            None => { would_block = true; },
            Some(packet) => return Ok(packet),
        }
    }
    Err(Error::TimedOut)
}

async fn transmit<F>(linkno: u8, f: F) -> Result<(), Error>
    where F: FnOnce(&mut [u8]) -> Result<usize, Error>
{
    let linkno = linkno as usize;
    unsafe {
        let _ = block_async!(tx_ready(linkno)).await;
        let ptr = DRTIOAUX_MEM[linkno].base as *mut u32;
        let len = DRTIOAUX_MEM[linkno].size / 2;
        // work buffer, works with unaligned mem access
        let mut buf: [u8; 1024] = [0; 1024]; 
        let len = f(&mut buf[0..len])?;
        copy_work_buffer(buf.as_mut_ptr() as *mut u32, ptr, len as isize);
        (DRTIOAUX[linkno].aux_tx_length_write)(len as u16);
        (DRTIOAUX[linkno].aux_tx_write)(1);
        Ok(())
    }
}

pub async fn send(linkno: u8, packet: &Packet) -> Result<(), Error> {
    transmit(linkno, |buffer| {
        let mut writer = Cursor::new(buffer);

        packet.write_to(&mut writer)?;
        
        // Pad till offset 4, insert checksum there
        let padding = (12 - (writer.position() % 8)) % 8;
        for _ in 0..padding {
            writer.write_u8(0)?;
        }

        let checksum = crc::crc32::checksum_ieee(&writer.get_ref()[0..writer.position()]);
        writer.write_u32(checksum)?;

        Ok(writer.position())
    }).await
}
