use crc;

use core_io::{ErrorKind as IoErrorKind, Error as IoError};
use io::{proto::ProtoRead, proto::ProtoWrite, Cursor};
use libboard_zynq::{timer::GlobalTimer, time::Milliseconds};
use crate::mem::mem::DRTIOAUX_MEM;
use crate::pl::csr::DRTIOAUX;
use crate::drtioaux_proto::Error as ProtocolError;

pub use crate::drtioaux_proto::Packet;

#[derive(Debug)]
pub enum Error {
    GatewareError,
    CorruptedPacket,

    LinkDown,
    TimedOut,
    UnexpectedReply,

    RoutingError,

    Protocol(ProtocolError)
}

impl From<ProtocolError> for Error {
    fn from(value: ProtocolError) -> Error {
        Error::Protocol(value)
    }
}

impl From<IoError> for Error {
    fn from(value: IoError) -> Error {
        Error::Protocol(ProtocolError::Io(value))
    }
}

pub fn reset(linkno: u8) {
    let linkno = linkno as usize;
    unsafe {
        // clear buffer first to limit race window with buffer overflow
        // error. We assume the CPU is fast enough so that no two packets
        // will be received between the buffer and the error flag are cleared.
        (DRTIOAUX[linkno].aux_rx_present_write)(1);
        (DRTIOAUX[linkno].aux_rx_error_write)(1);
    }
}

pub fn has_rx_error(linkno: u8) -> bool {
    let linkno = linkno as usize;
    unsafe {
        let error = (DRTIOAUX[linkno].aux_rx_error_read)() != 0;
        if error {
            (DRTIOAUX[linkno].aux_rx_error_write)(1)
        }
        error
    }
}

pub fn copy_work_buffer(src: *mut u16, dst: *mut u16, len: isize) {
    // AXI writes must be 4-byte aligned (drtio proto doesn't care for that),
    // and AXI burst reads/writes are not implemented yet in gateware
    // thus the need for a work buffer for transmitting and copying it over
    unsafe {
        for i in (0..(len/2)).step_by(2) {
            *dst.offset(i) = *src.offset(i);
            *dst.offset(i+1) = *src.offset(i+1);
        }
    }
}

fn receive<F, T>(linkno: u8, f: F) -> Result<Option<T>, Error>
    where F: FnOnce(&[u8]) -> Result<T, Error>
{
    let linkidx = linkno as usize;
    unsafe {
        if (DRTIOAUX[linkidx].aux_rx_present_read)() == 1 {
            let ptr = (DRTIOAUX_MEM[linkidx].base + DRTIOAUX_MEM[linkidx].size / 2) as *mut u16;
            let len = (DRTIOAUX[linkidx].aux_rx_length_read)() as usize;
            // work buffer to accomodate axi burst reads
            let mut buf: [u8; 1024] = [0; 1024];
            copy_work_buffer(ptr, buf.as_mut_ptr() as *mut u16, len as isize);
            let result = f(&buf[0..len]);
            (DRTIOAUX[linkidx].aux_rx_present_write)(1);
            Ok(Some(result?))
        } else {
            Ok(None)
        }
    }
}

pub fn recv(linkno: u8) -> Result<Option<Packet>, Error> {
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
    })
}

pub fn recv_timeout(linkno: u8, timeout_ms: Option<u64>,
    timer: GlobalTimer) -> Result<Packet, Error> 
{
    let timeout_ms = Milliseconds(timeout_ms.unwrap_or(10));
    let limit = timer.get_time() + timeout_ms;
    while timer.get_time() < limit {
        match recv(linkno)? {
            None => (),
            Some(packet) => return Ok(packet),
        }
    }
    Err(Error::TimedOut)
}

fn transmit<F>(linkno: u8, f: F) -> Result<(), Error>
    where F: FnOnce(&mut [u8]) -> Result<usize, Error>
{
    let linkno = linkno as usize;
    unsafe {
        while (DRTIOAUX[linkno].aux_tx_read)() != 0 {}
        let ptr = DRTIOAUX_MEM[linkno].base as *mut u16;
        let len = DRTIOAUX_MEM[linkno].size / 2;
        // work buffer, works with unaligned mem access
        let mut buf: [u8; 1024] = [0; 1024]; 
        let len = f(&mut buf[0..len])?;
        copy_work_buffer(buf.as_mut_ptr() as *mut u16, ptr, len as isize);
        (DRTIOAUX[linkno].aux_tx_length_write)(len as u16);
        (DRTIOAUX[linkno].aux_tx_write)(1);
        Ok(())
    }
}

pub fn send(linkno: u8, packet: &Packet) -> Result<(), Error> {
    transmit(linkno, |buffer| {
        let mut writer = Cursor::new(buffer);

        packet.write_to(&mut writer)?;
        
        let padding = 4 - (writer.position() % 4);
        if padding != 4 {
            for _ in 0..padding {
                writer.write_u8(0)?;
            }
        }

        let checksum = crc::crc32::checksum_ieee(&writer.get_ref()[0..writer.position()]);
        writer.write_u32(checksum)?;

        Ok(writer.position())
    })
}
