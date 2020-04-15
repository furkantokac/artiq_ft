use core::{mem::transmute, task::Poll};
use core::fmt;
use core::cmp::min;
use core::cell::RefCell;

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

use libboard_zynq::{
    println,
    self as zynq,
    smoltcp::{
        self,
        wire::{EthernetAddress, IpAddress, Ipv4Address, IpCidr},
        iface::{NeighborCache, EthernetInterfaceBuilder, Routes},
        time::Instant,
    },
};
use libsupport_zynq::alloc::{vec, vec::Vec};
use libcortex_a9::sync_channel;
use libasync::smoltcp::{Sockets, TcpStream};

use crate::kernel;


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


async fn expect(stream: &TcpStream, pattern: &[u8]) -> Result<()> {
    stream.recv(|buf| {
        for (i, b) in buf.iter().enumerate() {
            if *b == pattern[i] {
                if i + 1 == pattern.len() {
                    return Poll::Ready((i + 1, Ok(())));
                }
            } else {
                return Poll::Ready((i + 1, Err(Error::UnexpectedPattern)));
            }
        }
        Poll::Pending
    }).await?
}

async fn read_i8(stream: &TcpStream) -> Result<i8> {
    Ok(stream.recv(|buf| {
        Poll::Ready((1, buf[0] as i8))
    }).await?)
}

async fn read_i32(stream: &TcpStream) -> Result<i32> {
    Ok(stream.recv(|buf| {
        if buf.len() >= 4 {
            let value =
                  ((buf[0] as i32) << 24)
                | ((buf[1] as i32) << 16)
                | ((buf[2] as i32) << 8)
                |  (buf[3] as i32);
            Poll::Ready((4, value))
        } else {
            Poll::Pending
        }
    }).await?)
}

async fn read_drain(stream: &TcpStream, total: usize) -> Result<()> {
    let mut done = 0;
    while done < total {
        let count = stream.recv(|buf| {
            let count = min(total - done, buf.len());
            Poll::Ready((count, count))
        }).await?;
        done += count;
    }
    Ok(())
}

async fn read_chunk(stream: &TcpStream, destination: &mut [u8]) -> Result<()> {
    let total = destination.len();
    let destination = RefCell::new(destination);
    let mut done = 0;
    while done < total {
        let count = stream.recv(|buf| {
            let mut destination = destination.borrow_mut();
            let count = min(total - done, buf.len());
            destination[done..done + count].copy_from_slice(&buf[..count]);
            Poll::Ready((count, count))
        }).await?;
        done += count;
    }
    Ok(())
}

#[derive(FromPrimitive, ToPrimitive)]
enum Request {
    SystemInfo = 3,
    LoadKernel = 5,
    RunKernel = 6,
    RPCReply = 7,
    RPCException = 8,
}

#[derive(FromPrimitive, ToPrimitive)]
enum Reply {
    SystemInfo = 2,
    LoadCompleted = 5,
    LoadFailed = 6,
    KernelFinished = 7,
    KernelStartupFailed = 8,
    KernelException = 9,
    RPCRequest = 10,
    WatchdogExpired = 14,
    ClockFailure = 15,
}

async fn send_header(stream: &TcpStream, reply: Reply) -> Result<()> {
    stream.send([0x5a, 0x5a, 0x5a, 0x5a, reply.to_u8().unwrap()].iter().copied()).await?;
    Ok(())
}

async fn handle_connection(stream: TcpStream) -> Result<()> {
    expect(&stream, b"ARTIQ coredev\n").await?;
    loop {
        expect(&stream, &[0x5a, 0x5a, 0x5a, 0x5a]).await?;
        let request: Request = FromPrimitive::from_i8(read_i8(&stream).await?)
            .ok_or(Error::UnrecognizedPacket)?;
        match request {
            Request::SystemInfo => {
                send_header(&stream, Reply::SystemInfo).await?;
                stream.send("ARZQ".bytes()).await?;
            },
            Request::LoadKernel => {
                let length = read_i32(&stream).await? as usize;
                let mut kernel_buffer = unsafe { &mut kernel::KERNEL_BUFFER };
                if kernel_buffer.len() < length {
                    read_drain(&stream, length).await?;
                    send_header(&stream, Reply::LoadFailed).await?;
                } else {
                    read_chunk(&stream, &mut kernel_buffer[..length]).await?;
                    send_header(&stream, Reply::LoadCompleted).await?;
                }
                println!("length={}, {:?}", length, &kernel_buffer[..256]);
            }
            _ => return Err(Error::UnrecognizedPacket)
        }
    }
}


const HWADDR: [u8; 6] = [0, 0x23, 0xab, 0xad, 0x1d, 0xea];
const IPADDR: IpAddress = IpAddress::Ipv4(Ipv4Address([192, 168, 1, 52]));

pub fn main(mut sc_tx: sync_channel::Sender<usize>, mut sc_rx: sync_channel::Receiver<usize>) {
    let eth = zynq::eth::Eth::default(HWADDR.clone());
    const RX_LEN: usize = 8;
    let mut rx_descs = (0..RX_LEN)
        .map(|_| zynq::eth::rx::DescEntry::zeroed())
        .collect::<Vec<_>>();
    let mut rx_buffers = vec![zynq::eth::Buffer::new(); RX_LEN];
    // Number of transmission buffers (minimum is two because with
    // one, duplicate packet transmission occurs)
    const TX_LEN: usize = 8;
    let mut tx_descs = (0..TX_LEN)
        .map(|_| zynq::eth::tx::DescEntry::zeroed())
        .collect::<Vec<_>>();
    let mut tx_buffers = vec![zynq::eth::Buffer::new(); TX_LEN];
    let eth = eth.start_rx(&mut rx_descs, &mut rx_buffers);
    let mut eth = eth.start_tx(
        // HACK
        unsafe { transmute(tx_descs.as_mut_slice()) },
        unsafe { transmute(tx_buffers.as_mut_slice()) },
    );
    let ethernet_addr = EthernetAddress(HWADDR);

    let mut ip_addrs = [IpCidr::new(IPADDR, 24)];
    let mut routes_storage = vec![None; 4];
    let routes = Routes::new(&mut routes_storage[..]);
    let mut neighbor_storage = vec![None; 256];
    let neighbor_cache = NeighborCache::new(&mut neighbor_storage[..]);
    let mut iface = EthernetInterfaceBuilder::new(&mut eth)
        .ethernet_addr(ethernet_addr)
        .ip_addrs(&mut ip_addrs[..])
        .routes(routes)
        .neighbor_cache(neighbor_cache)
        .finalize();

    Sockets::init(32);

    TcpStream::listen(1381, 2048, 2048, 8, |stream| async {
        let _ = handle_connection(stream)
            .await
            .map_err(|e| println!("Connection: {}", e));
    });

    let mut time = 0u32;
    Sockets::run(&mut iface, || {
        time += 1;
        Instant::from_millis(time)
    });
}
