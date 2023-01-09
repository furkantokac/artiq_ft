use core::cell::RefCell;
use alloc::rc::Rc;
use alloc::collections::BTreeMap;
use alloc::string::String;
use libboard_zynq::timer::GlobalTimer;
use libboard_artiq::{pl::csr, drtio_routing};
use libcortex_a9::mutex::Mutex;
use libconfig::Config;
use io::{Cursor, ProtoRead};
use log::error;

static mut RTIO_DEVICE_MAP: BTreeMap<u32, String> = BTreeMap::new();


#[cfg(has_drtio)]
pub mod drtio {
    use super::*;
    use crate::{SEEN_ASYNC_ERRORS, ASYNC_ERROR_BUSY, ASYNC_ERROR_SEQUENCE_ERROR, ASYNC_ERROR_COLLISION};
    use libboard_artiq::drtioaux_async;
    use libboard_artiq::drtioaux_async::Packet;
    use libboard_artiq::drtioaux::Error;
    use log::{warn, error, info};
    use embedded_hal::blocking::delay::DelayMs;
    use libasync::{task, delay};
    use libboard_zynq::time::Milliseconds;

    pub fn startup(aux_mutex: &Rc<Mutex<bool>>,
            routing_table: &Rc<RefCell<drtio_routing::RoutingTable>>,
            up_destinations: &Rc<RefCell<[bool; drtio_routing::DEST_COUNT]>>,
            timer: GlobalTimer) {
        let aux_mutex = aux_mutex.clone();
        let routing_table = routing_table.clone();
        let up_destinations = up_destinations.clone();
        task::spawn(async move {
            let routing_table = routing_table.borrow();
            link_task(&aux_mutex, &routing_table, &up_destinations, timer).await;
        });
    }

    async fn link_rx_up(linkno: u8) -> bool {
        let linkno = linkno as usize;
        unsafe {
            (csr::DRTIO[linkno].rx_up_read)() == 1
        }
    }

    async fn recv_aux_timeout(linkno: u8, timeout: u64, timer: GlobalTimer) -> Result<Packet, &'static str> {
        if !link_rx_up(linkno).await {
            return Err("link went down");
        }
        match drtioaux_async::recv_timeout(linkno, Some(timeout), timer).await {
            Ok(packet) => return Ok(packet),
            Err(Error::TimedOut) => return Err("timed out"),
            Err(_) => return Err("aux packet error"),
        }
    }

    pub async fn aux_transact(aux_mutex: &Mutex<bool>, linkno: u8, request: &Packet,
            timer: GlobalTimer) -> Result<Packet, &'static str> {
        if !link_rx_up(linkno).await {
            return Err("link went down");
        }
        let _lock = aux_mutex.async_lock().await;
        drtioaux_async::send(linkno, request).await.unwrap();
        recv_aux_timeout(linkno, 200, timer).await
    }

    async fn drain_buffer(linkno: u8, draining_time: Milliseconds, timer: GlobalTimer) {
        let max_time = timer.get_time() + draining_time;
        loop {
            if timer.get_time() > max_time {
                return;
            } //could this be cut short?
            let _ = drtioaux_async::recv(linkno).await;
        }
    }

    async fn ping_remote(aux_mutex: &Rc<Mutex<bool>>, linkno: u8, timer: GlobalTimer) -> u32 {
        let mut count = 0;
        loop {
            if !link_rx_up(linkno).await {
                return 0
            }
            count += 1;
            if count > 100 {
                return 0;
            }
            let reply = aux_transact(aux_mutex, linkno, &Packet::EchoRequest, timer).await;
            match reply {
                Ok(Packet::EchoReply) => {
                    // make sure receive buffer is drained
                    let draining_time = Milliseconds(200);
                    drain_buffer(linkno, draining_time, timer).await;
                    return count;
                }
                _ => {}
            }
        }
    }

    async fn sync_tsc(aux_mutex: &Rc<Mutex<bool>>, linkno: u8, timer: GlobalTimer) -> Result<(), &'static str> {
        let _lock = aux_mutex.async_lock().await;

        unsafe {
            (csr::DRTIO[linkno as usize].set_time_write)(1);
            while (csr::DRTIO[linkno as usize].set_time_read)() == 1 {}
        }
        // TSCAck is the only aux packet that is sent spontaneously
        // by the satellite, in response to a TSC set on the RT link.
        let reply = recv_aux_timeout(linkno, 10000, timer).await?;
        if reply == Packet::TSCAck {
            return Ok(());
        } else {
            return Err("unexpected reply");
        }
    }

    async fn load_routing_table(aux_mutex: &Rc<Mutex<bool>>, linkno: u8, routing_table: &drtio_routing::RoutingTable,
        timer: GlobalTimer) -> Result<(), &'static str> {
        for i in 0..drtio_routing::DEST_COUNT {
            let reply = aux_transact(aux_mutex, linkno, &Packet::RoutingSetPath {
                destination: i as u8,
                hops: routing_table.0[i]
            }, timer).await?;
            if reply != Packet::RoutingAck {
                return Err("unexpected reply");
            }
        }
        Ok(())
    }

    async fn set_rank(aux_mutex: &Rc<Mutex<bool>>, linkno: u8, rank: u8, timer: GlobalTimer) -> Result<(), &'static str> {
        let reply = aux_transact(aux_mutex, linkno, &Packet::RoutingSetRank {
            rank: rank
        }, timer).await?;
        if reply != Packet::RoutingAck {
            return Err("unexpected reply");
        }
        Ok(())
    }

    async fn init_buffer_space(destination: u8, linkno: u8) {
        let linkno = linkno as usize;
        unsafe {
            (csr::DRTIO[linkno].destination_write)(destination);
            (csr::DRTIO[linkno].force_destination_write)(1);
            (csr::DRTIO[linkno].o_get_buffer_space_write)(1);
            while (csr::DRTIO[linkno].o_wait_read)() == 1 {}
            info!("[DEST#{}] buffer space is {}",
                destination, (csr::DRTIO[linkno].o_dbg_buffer_space_read)());
            (csr::DRTIO[linkno].force_destination_write)(0);
        }
    }

    async fn process_unsolicited_aux(aux_mutex: &Rc<Mutex<bool>>, linkno: u8) {
        let _lock = aux_mutex.async_lock().await;
        match drtioaux_async::recv(linkno).await {
            Ok(Some(packet)) => warn!("[LINK#{}] unsolicited aux packet: {:?}", linkno, packet),
            Ok(None) => (),
            Err(_) => warn!("[LINK#{}] aux packet error", linkno)
        }
    }

    async fn process_local_errors(linkno: u8) {
        let errors;
        let linkidx = linkno as usize;
        unsafe {
            errors = (csr::DRTIO[linkidx].protocol_error_read)();
            (csr::DRTIO[linkidx].protocol_error_write)(errors);
        }
        if errors != 0 {
            error!("[LINK#{}] error(s) found (0x{:02x}):", linkno, errors);
            if errors & 1 != 0 {
                error!("[LINK#{}] received packet of an unknown type", linkno);
            }
            if errors & 2 != 0 {
                error!("[LINK#{}] received truncated packet", linkno);
            }
            if errors & 4 != 0 {
                error!("[LINK#{}] timeout attempting to get remote buffer space", linkno);
            }
        }
    }

    async fn destination_set_up(routing_table: &drtio_routing::RoutingTable,
            up_destinations: &Rc<RefCell<[bool; drtio_routing::DEST_COUNT]>>,
            destination: u8, up: bool) {
        let mut up_destinations = up_destinations.borrow_mut();
        up_destinations[destination as usize] = up;
        if up {
            drtio_routing::interconnect_enable(routing_table, 0, destination);
            info!("[DEST#{}] destination is up", destination);
        } else {
            drtio_routing::interconnect_disable(destination);
            info!("[DEST#{}] destination is down", destination);
        }
    }

    async fn destination_up(up_destinations: &Rc<RefCell<[bool; drtio_routing::DEST_COUNT]>>, destination: u8) -> bool {
        let up_destinations = up_destinations.borrow();
        up_destinations[destination as usize]
    }

    async fn destination_survey(aux_mutex: &Rc<Mutex<bool>>, routing_table: &drtio_routing::RoutingTable,
            up_links: &[bool],
            up_destinations: &Rc<RefCell<[bool; drtio_routing::DEST_COUNT]>>,
            timer: GlobalTimer) {
        for destination in 0..drtio_routing::DEST_COUNT {
            let hop = routing_table.0[destination][0];
            let destination = destination as u8;

            if hop == 0 {
                /* local RTIO */
                if !destination_up(up_destinations, destination).await {
                    destination_set_up(routing_table, up_destinations, destination, true).await;
                }
            } else if hop as usize <= csr::DRTIO.len() {
                let linkno = hop - 1;
                if destination_up(up_destinations, destination).await {
                    if up_links[linkno as usize] {
                        let reply = aux_transact(aux_mutex, linkno, &Packet::DestinationStatusRequest {
                            destination: destination
                        }, timer).await;
                        match reply {
                            Ok(Packet::DestinationDownReply) =>
                                destination_set_up(routing_table, up_destinations, destination, false).await,
                            Ok(Packet::DestinationOkReply) => (),
                            Ok(Packet::DestinationSequenceErrorReply { channel }) =>{
                                error!("[DEST#{}] RTIO sequence error involving channel {} 0x{:04x}", destination, resolve_channel_name(channel), channel);
                                unsafe { SEEN_ASYNC_ERRORS |= ASYNC_ERROR_SEQUENCE_ERROR };
                            }
                            Ok(Packet::DestinationCollisionReply { channel }) =>{
                                error!("[DEST#{}] RTIO collision involving channel {} 0x{:04x}", destination, resolve_channel_name(channel), channel);
                                unsafe { SEEN_ASYNC_ERRORS |= ASYNC_ERROR_COLLISION };
                            }
                            Ok(Packet::DestinationBusyReply { channel }) =>{
                                error!("[DEST#{}] RTIO busy error involving channel {} 0x{:04x}", destination, resolve_channel_name(channel), channel);
                                unsafe { SEEN_ASYNC_ERRORS |= ASYNC_ERROR_BUSY };
                            }
                            Ok(packet) => error!("[DEST#{}] received unexpected aux packet: {:?}", destination, packet),
                            Err(e) => error!("[DEST#{}] communication failed ({})", destination, e)
                        }
                    } else {
                        destination_set_up(routing_table, up_destinations, destination, false).await;
                    }
                } else {
                    if up_links[linkno as usize] {
                        let reply = aux_transact(aux_mutex, linkno, &Packet::DestinationStatusRequest {
                            destination: destination
                        }, timer).await;
                        match reply {
                            Ok(Packet::DestinationDownReply) => (),
                            Ok(Packet::DestinationOkReply) => {
                                destination_set_up(routing_table, up_destinations, destination, true).await;
                                init_buffer_space(destination as u8, linkno).await;
                            },
                            Ok(packet) => error!("[DEST#{}] received unexpected aux packet: {:?}", destination, packet),
                            Err(e) => error!("[DEST#{}] communication failed ({})", destination, e)
                        }
                    }
                }
            }
        }
    }

    pub async fn link_task(aux_mutex: &Rc<Mutex<bool>>,
            routing_table: &drtio_routing::RoutingTable,
            up_destinations: &Rc<RefCell<[bool; drtio_routing::DEST_COUNT]>>,
            timer: GlobalTimer) {
        let mut up_links = [false; csr::DRTIO.len()];
        loop {
            for linkno in 0..csr::DRTIO.len() {
                let linkno = linkno as u8;
                if up_links[linkno as usize] {
                    /* link was previously up */
                    if link_rx_up(linkno).await {
                        process_unsolicited_aux(aux_mutex, linkno).await;
                        process_local_errors(linkno).await;
                    } else {
                        info!("[LINK#{}] link is down", linkno);
                        up_links[linkno as usize] = false;
                    }
                } else {
                    /* link was previously down */
                    if link_rx_up(linkno).await {
                        info!("[LINK#{}] link RX became up, pinging", linkno);
                        let ping_count = ping_remote(aux_mutex, linkno, timer).await;
                        if ping_count > 0 {
                            info!("[LINK#{}] remote replied after {} packets", linkno, ping_count);
                            up_links[linkno as usize] = true;
                            if let Err(e) = sync_tsc(aux_mutex, linkno, timer).await {
                                error!("[LINK#{}] failed to sync TSC ({})", linkno, e);
                            }
                            if let Err(e) = load_routing_table(aux_mutex, linkno, routing_table, timer).await {
                                error!("[LINK#{}] failed to load routing table ({})", linkno, e);
                            }
                            if let Err(e) = set_rank(aux_mutex, linkno, 1 as u8, timer).await {
                                error!("[LINK#{}] failed to set rank ({})", linkno, e);
                            }
                            info!("[LINK#{}] link initialization completed", linkno);
                        } else {
                            error!("[LINK#{}] ping failed", linkno);
                        }
                    }
                }
            }
            destination_survey(aux_mutex, routing_table, &up_links, up_destinations, timer).await;
            let mut countdown = timer.countdown();
            delay(&mut countdown, Milliseconds(200)).await;
        }
    }

    #[allow(dead_code)]
    pub fn reset(aux_mutex: Rc<Mutex<bool>>, mut timer: GlobalTimer) {
        for linkno in 0..csr::DRTIO.len() {
            unsafe {
                (csr::DRTIO[linkno].reset_write)(1);
            }
        }
        timer.delay_ms(1);
        for linkno in 0..csr::DRTIO.len() {
            unsafe {
                (csr::DRTIO[linkno].reset_write)(0);
            }
        }

        for linkno in 0..csr::DRTIO.len() {
            let linkno = linkno as u8;
            if task::block_on(link_rx_up(linkno)) {
                let reply = task::block_on(aux_transact(&aux_mutex, linkno,
                    &Packet::ResetRequest, timer));
                match reply {
                    Ok(Packet::ResetAck) => (),
                    Ok(_) => error!("[LINK#{}] reset failed, received unexpected aux packet", linkno),
                    Err(e) => error!("[LINK#{}] reset failed, aux packet error ({})", linkno, e)
                }
            }
        }
    }
}

fn read_device_map(cfg: &Config) -> BTreeMap<u32, String> {
    let mut device_map: BTreeMap<u32, String> = BTreeMap::new();
    let _ = cfg.read("device_map").and_then(|raw_bytes| {
        let mut bytes_cr = Cursor::new(raw_bytes);
        let size = bytes_cr.read_u32().unwrap();
        for _ in 0..size {
            let channel = bytes_cr.read_u32().unwrap();
            let device_name = bytes_cr.read_string().unwrap();
            if let Some(old_entry) = device_map.insert(channel, device_name.clone()) {
                error!("read_device_map: conflicting entries for channel {}: `{}` and `{}`",
                       channel, old_entry, device_name);
            }
        }
        Ok(())
    } ).or_else(|err| {
        error!("read_device_map: error reading `device_map` from config: {}", err);
        Err(err)
    });
    device_map
}

fn _resolve_channel_name(channel: u32, device_map: &BTreeMap<u32, String>) -> String {
    match device_map.get(&channel) {
        Some(val) => val.clone(),
        None => String::from("unknown")
    }
}

pub fn resolve_channel_name(channel: u32) -> String {
    _resolve_channel_name(channel, unsafe{&RTIO_DEVICE_MAP})
}

#[cfg(not(has_drtio))]
pub mod drtio {
    use super::*;

    pub fn startup(_aux_mutex: &Rc<Mutex<bool>>, _routing_table: &Rc<RefCell<drtio_routing::RoutingTable>>,
        _up_destinations: &Rc<RefCell<[bool; drtio_routing::DEST_COUNT]>>, _timer: GlobalTimer) {}
    
    #[allow(dead_code)]
    pub fn reset(_aux_mutex: Rc<Mutex<bool>>, mut _timer: GlobalTimer) {}
}

pub fn startup(aux_mutex: &Rc<Mutex<bool>>,
        routing_table: &Rc<RefCell<drtio_routing::RoutingTable>>,
        up_destinations: &Rc<RefCell<[bool; drtio_routing::DEST_COUNT]>>, 
        timer: GlobalTimer,
        cfg: &Config) {
    unsafe { RTIO_DEVICE_MAP = read_device_map(cfg); }
    drtio::startup(aux_mutex, routing_table, up_destinations, timer);
    unsafe {
        csr::rtio_core::reset_phy_write(1);
    }
}

#[allow(dead_code)]
pub fn reset(aux_mutex: Rc<Mutex<bool>>, timer: GlobalTimer) {
    unsafe {
        csr::rtio_core::reset_write(1);
    }
    drtio::reset(aux_mutex, timer)
}
