#![no_std]
#![no_main]

extern crate alloc;

use core::{cmp, str};
use core::mem::transmute;

use libboard_zynq::{
    println,
    self as zynq, clocks::Clocks, clocks::source::{ClockSource, ArmPll, IoPll},
    smoltcp::{
        wire::{EthernetAddress, IpAddress, IpCidr},
        iface::{NeighborCache, EthernetInterfaceBuilder, Routes},
        time::Instant,
    },
};
use libsupport_zynq::{
    ram, alloc::{vec, vec::Vec},
};
use libasync::smoltcp::Sockets;

mod pl;

fn identifier_read(buf: &mut [u8]) -> &str {
    unsafe {
        pl::csr::identifier::address_write(0);
        let len = pl::csr::identifier::data_read();
        let len = cmp::min(len, buf.len() as u8);
        for i in 0..len {
            pl::csr::identifier::address_write(1 + i);
            buf[i as usize] = pl::csr::identifier::data_read();
        }
        str::from_utf8_unchecked(&buf[..len as usize])
    }
}

const HWADDR: [u8; 6] = [0, 0x23, 0xab, 0xad, 0x1d, 0xea];

#[no_mangle]
pub fn main_core0() {
    println!("ARTIQ runtime starting...");

    const CPU_FREQ: u32 = 800_000_000;

    ArmPll::setup(2 * CPU_FREQ);
    Clocks::set_cpu_freq(CPU_FREQ);
    IoPll::setup(1_000_000_000);
    libboard_zynq::stdio::drop_uart(); // why?
    let mut ddr = zynq::ddr::DdrRam::new();
    ram::init_alloc(&mut ddr);

    println!("Detected gateware: {}", identifier_read(&mut [0; 64]));

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

    let local_addr = IpAddress::v4(192, 168, 1, 52);
    let mut ip_addrs = [IpCidr::new(local_addr, 24)];
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
    let mut time = 0u32;
    Sockets::run(&mut iface, || {
        time += 1;
        Instant::from_millis(time)
    });
}

#[no_mangle]
pub fn main_core1() {
    println!("[CORE1] hello world {}", identifier_read(&mut [0; 64]));
    loop {}
}
