use core::{mem::transmute, task::Poll};
use core::fmt;

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

use libasync::smoltcp::{Sockets, TcpStream};


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

async fn read_u8(stream: &TcpStream) -> Result<u8> {
    Ok(stream.recv(|buf| {
        if buf.len() >= 1 {
            Poll::Ready((1, buf[0]))
        } else {
            Poll::Pending
        }
    }).await?)
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
        let request: Request = FromPrimitive::from_u8(read_u8(&stream).await?)
            .ok_or(Error::UnrecognizedPacket)?;
        match request {
            Request::SystemInfo => {
                send_header(&stream, Reply::SystemInfo).await?;
                stream.send("ARZQ".bytes()).await?;
            },
            _ => return Err(Error::UnrecognizedPacket)
        }
    }
}


const HWADDR: [u8; 6] = [0, 0x23, 0xab, 0xad, 0x1d, 0xea];
const IPADDR: IpAddress = IpAddress::Ipv4(Ipv4Address([192, 168, 1, 52]));

pub fn network_main() {
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
