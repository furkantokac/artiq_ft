use core::{mem::transmute, task::Poll};
use core::fmt;
use core::cmp::min;
use core::cell::RefCell;
use alloc::rc::Rc;
use log::{warn, error};

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

use libboard_zynq::{
    self as zynq,
    smoltcp::{
        self,
        wire::{EthernetAddress, IpAddress, Ipv4Address, IpCidr},
        iface::{NeighborCache, EthernetInterfaceBuilder, Routes},
        time::Instant,
    },
    timer::GlobalTimer,
};
use libsupport_zynq::alloc::{vec, vec::Vec};
use libasync::{smoltcp::{Sockets, TcpStream}, task};
use alloc::sync::Arc;

use crate::proto::*;
use crate::kernel;
use crate::moninj;


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

#[derive(Debug, FromPrimitive, ToPrimitive)]
enum Request {
    SystemInfo = 3,
    LoadKernel = 5,
    RunKernel = 6,
    RPCReply = 7,
    RPCException = 8,
}

#[derive(Debug, FromPrimitive, ToPrimitive)]
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

async fn write_chunk(stream: &TcpStream, chunk: &[u8]) -> Result<()> {
    write_i32(stream, chunk.len() as i32).await?;
    stream.send(chunk.iter().copied()).await?;
    Ok(())
}

async fn write_header(stream: &TcpStream, reply: Reply) -> Result<()> {
    stream.send([0x5a, 0x5a, 0x5a, 0x5a, reply.to_u8().unwrap()].iter().copied()).await?;
    Ok(())
}

async fn handle_connection(stream: &TcpStream, control: Rc<RefCell<kernel::Control>>) -> Result<()> {
    expect(&stream, b"ARTIQ coredev\n").await?;
    loop {
        if !expect(&stream, &[0x5a, 0x5a, 0x5a, 0x5a]).await? {
            return Err(Error::UnexpectedPattern)
        }
        let request: Request = FromPrimitive::from_i8(read_i8(&stream).await?)
            .ok_or(Error::UnrecognizedPacket)?;
        match request {
            Request::SystemInfo => {
                write_header(&stream, Reply::SystemInfo).await?;
                stream.send("ARZQ".bytes()).await?;
            },
            Request::LoadKernel => {
                let length = read_i32(&stream).await? as usize;
                if length < 1024*1024 {
                    let mut buffer = vec![0; length];
                    read_chunk(&stream, &mut buffer).await?;

                    let mut control = control.borrow_mut();
                    control.restart();
                    control.tx.async_send(kernel::Message::LoadRequest(Arc::new(buffer))).await;
                    let reply = control.rx.async_recv().await;
                    match *reply {
                        kernel::Message::LoadCompleted => write_header(&stream, Reply::LoadCompleted).await?,
                        kernel::Message::LoadFailed => {
                            write_header(&stream, Reply::LoadFailed).await?;
                            write_chunk(&stream, b"core1 failed to process data").await?;
                        },
                        _ => {
                            error!("received unexpected message from core1: {:?}", reply);
                            write_header(&stream, Reply::LoadFailed).await?;
                            write_chunk(&stream, b"core1 sent unexpected reply").await?;
                        }
                    }
                } else {
                    read_drain(&stream, length).await?;
                    write_header(&stream, Reply::LoadFailed).await?;
                    write_chunk(&stream, b"kernel is too large").await?;
                }
            },
            Request::RunKernel => {
                let mut control = control.borrow_mut();
                control.tx.async_send(kernel::Message::StartRequest).await;
            }
            _ => return Err(Error::UnrecognizedPacket)
        }
    }
}


const HWADDR: [u8; 6] = [0, 0x23, 0xab, 0xad, 0x1d, 0xea];
const IPADDR: IpAddress = IpAddress::Ipv4(Ipv4Address([192, 168, 1, 52]));

pub fn main(timer: GlobalTimer) {
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

    let control: Rc<RefCell<kernel::Control>> = Rc::new(RefCell::new(kernel::Control::start(8192)));
    task::spawn(async move {
        loop {
            let stream = TcpStream::accept(1381, 2048, 2048).await.unwrap();
            let control = control.clone();
            task::spawn(async {
                let _ = handle_connection(&stream, control)
                    .await
                    .map_err(|e| warn!("connection terminated: {}", e));
                let _ = stream.flush().await;
                let _ = stream.abort().await;
            });
        }
    });

    moninj::start(timer);

    Sockets::run(&mut iface, || {
        Instant::from_millis(timer.get_time().0 as i32)
    });
}
