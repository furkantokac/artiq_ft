use core::{mem::transmute, task::Poll};

use alloc::{borrow::ToOwned, format};
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


async fn handle_connection(stream: TcpStream) -> smoltcp::Result<()> {
    stream.send("Enter your name: ".bytes()).await?;
    let name = stream.recv(|buf| {
        for (i, b) in buf.iter().enumerate() {
            if *b == '\n' as u8 {
                return match core::str::from_utf8(&buf[0..i]) {
                    Ok(name) =>
                        Poll::Ready((i + 1, Some(name.to_owned()))),
                    Err(_) =>
                        Poll::Ready((i + 1, None))
                };
            }
        }
        if buf.len() > 100 {
            // Too much input, consume all
            Poll::Ready((buf.len(), None))
        } else {
            Poll::Pending
        }
    }).await?;
    match name {
        Some(name) =>
            stream.send(format!("Hello {}!\n", name).bytes()).await?,
        None =>
            stream.send("I had trouble reading your name.\n".bytes()).await?,
    }
    stream.flush().await;
    Ok(())
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
            .map_err(|e| println!("Connection: {:?}", e));
    });

    let mut time = 0u32;
    Sockets::run(&mut iface, || {
        time += 1;
        Instant::from_millis(time)
    });
}
