use libasync::{smoltcp::TcpStream, task};
use libboard_zynq::smoltcp::Error;
use libcortex_a9::cache;
use log::{debug, info, warn};

use crate::proto_async::*;
use crate::pl;

const BUFFER_SIZE: usize = 512 * 1024;

#[repr(align(64))]
struct Buffer {
    data: [u8; BUFFER_SIZE],
}

static mut BUFFER: Buffer = Buffer {
    data: [0; BUFFER_SIZE]
};

fn arm() {
    debug!("arming RTIO analyzer");
    unsafe {
        let base_addr = &mut BUFFER.data[0] as *mut _ as usize;
        let last_addr = &mut BUFFER.data[BUFFER_SIZE - 1] as *mut _ as usize;
        pl::csr::rtio_analyzer::message_encoder_overflow_reset_write(1);
        pl::csr::rtio_analyzer::dma_base_address_write(base_addr as u32);
        pl::csr::rtio_analyzer::dma_last_address_write(last_addr as u32);
        pl::csr::rtio_analyzer::dma_reset_write(1);
        pl::csr::rtio_analyzer::enable_write(1);
    }
}

fn disarm() {
    debug!("disarming RTIO analyzer");
    unsafe {
        pl::csr::rtio_analyzer::enable_write(0);
        while pl::csr::rtio_analyzer::busy_read() != 0 {}
        cache::dcci_slice(&BUFFER.data);
    }
    debug!("RTIO analyzer disarmed");
}

#[derive(Debug)]
struct Header {
    sent_bytes: u32,
    total_byte_count: u64,
    error_occurred: bool,
    log_channel: u8,
    dds_onehot_sel: bool
}

async fn write_header(stream: &mut TcpStream, header: &Header) -> Result<(), Error> {
    stream.send_slice("e".as_bytes()).await?;
    write_i32(stream, header.sent_bytes as i32).await?;
    write_i64(stream, header.total_byte_count as i64).await?;
    write_i8(stream, header.error_occurred as i8).await?;
    write_i8(stream, header.log_channel as i8).await?;
    write_i8(stream, header.dds_onehot_sel as i8).await?;
    Ok(())
}

async fn handle_connection(stream: &mut TcpStream) -> Result<(), Error> {
    info!("received connection");

    let data = unsafe { &BUFFER.data[..] };
    let overflow_occurred = unsafe { pl::csr::rtio_analyzer::message_encoder_overflow_read() != 0 };
    let bus_error_occurred = unsafe { pl::csr::rtio_analyzer::dma_bus_error_read() != 0 };
    let total_byte_count = unsafe { pl::csr::rtio_analyzer::dma_byte_count_read() as u64 };
    let pointer = (total_byte_count % BUFFER_SIZE as u64) as usize;
    let wraparound = total_byte_count >= BUFFER_SIZE as u64;

    if overflow_occurred {
        warn!("overflow occured");
    }
    if bus_error_occurred {
        warn!("bus error occured");
    }

    let header = Header {
        total_byte_count: total_byte_count,
        sent_bytes: if wraparound { BUFFER_SIZE as u32 } else { total_byte_count as u32 },
        error_occurred: overflow_occurred | bus_error_occurred,
        log_channel: pl::csr::CONFIG_RTIO_LOG_CHANNEL as u8,
        dds_onehot_sel: true  // kept for backward compatibility of analyzer dumps
    };
    debug!("{:?}", header);

    write_header(stream, &header).await?;
    if wraparound {
        stream.send(data[pointer..].iter().copied()).await?;
        stream.send(data[..pointer].iter().copied()).await?;
    } else {
        stream.send(data[..pointer].iter().copied()).await?;
    }

    Ok(())
}

pub fn start() {
    task::spawn(async move {
        loop {
            arm();
            let mut stream = TcpStream::accept(1382, 2048, 2048).await.unwrap();
            disarm();
            let _ = handle_connection(&mut stream)
                .await
                .map_err(|e| warn!("connection terminated: {:?}", e));
            let _ = stream.flush().await;
            let _ = stream.close().await;
        }
    });
}
