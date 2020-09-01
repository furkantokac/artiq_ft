use alloc::vec;
use alloc::vec::Vec;
use byteorder::{ByteOrder, NetworkEndian};
use core_io::{Read, Seek};
use libboard_zynq::{
    devc,
    eth::Eth,
    smoltcp::{
        self,
        iface::{EthernetInterfaceBuilder, NeighborCache},
        time::Instant,
        wire::IpCidr,
    },
    timer::GlobalTimer,
};
use libconfig::{bootgen, net_settings, Config};

enum NetConnState {
    WaitCommand,
    FirmwareLength(usize, u8),
    FirmwareDownload(usize, usize),
    FirmwareWaitO,
    FirmwareWaitK,
    GatewareLength(usize, u8),
    GatewareDownload(usize, usize),
    GatewareWaitO,
    GatewareWaitK,
}

struct NetConn {
    state: NetConnState,
    firmware_downloaded: bool,
    gateware_downloaded: bool,
}

impl NetConn {
    pub fn new() -> NetConn {
        NetConn {
            state: NetConnState::WaitCommand,
            firmware_downloaded: false,
            gateware_downloaded: false,
        }
    }

    pub fn reset(&mut self) {
        self.state = NetConnState::WaitCommand;
        self.firmware_downloaded = false;
        self.gateware_downloaded = false;
    }

    fn input_partial<File: Read + Seek>(
        &mut self,
        bootgen_file: &mut Option<File>,
        runtime_start: *mut u8,
        runtime_max_len: usize,
        buf: &[u8],
        storage: &mut Vec<u8>,
        mut boot_callback: impl FnMut(),
    ) -> Result<usize, ()> {
        match self.state {
            NetConnState::WaitCommand => match buf[0] {
                b'F' => {
                    log::info!("Received firmware load command");
                    self.state = NetConnState::FirmwareLength(0, 0);
                    Ok(1)
                }
                b'G' => {
                    log::info!("Received gateware load command");
                    self.state = NetConnState::GatewareLength(0, 0);
                    storage.clear();
                    Ok(1)
                }
                b'B' => {
                    if !self.gateware_downloaded {
                        log::info!("Gateware not loaded via netboot");
                        if bootgen_file.is_none() {
                            log::error!("No bootgen file to load gateware");
                            return Err(());
                        }
                        log::info!("Attempting to load from SD card");
                        if let Err(e) = bootgen::load_bitstream(bootgen_file.as_mut().unwrap()) {
                            log::error!("Gateware load failed: {:?}", e);
                            return Err(());
                        }
                    }
                    if self.firmware_downloaded {
                        log::info!("Received boot command");
                        boot_callback();
                        self.state = NetConnState::WaitCommand;
                        Ok(1)
                    } else {
                        log::error!("Received boot command, but no firmware downloaded");
                        Err(())
                    }
                }
                _ => {
                    log::error!("Received unknown netboot command: 0x{:02x}", buf[0]);
                    Err(())
                }
            },
            NetConnState::FirmwareLength(firmware_length, recv_bytes) => {
                let firmware_length = (firmware_length << 8) | (buf[0] as usize);
                let recv_bytes = recv_bytes + 1;
                if recv_bytes == 4 {
                    if firmware_length > runtime_max_len {
                        log::error!(
                            "Runtime too large, maximum {} but requested {}",
                            runtime_max_len,
                            firmware_length
                        );
                        return Err(());
                    }
                    self.state = NetConnState::FirmwareDownload(firmware_length, 0);
                    storage.clear();
                    storage.reserve(firmware_length);
                } else {
                    self.state = NetConnState::FirmwareLength(firmware_length, recv_bytes);
                }
                Ok(1)
            }
            NetConnState::FirmwareDownload(firmware_length, recv_bytes) => {
                let max_length = firmware_length - recv_bytes;
                let buf = if buf.len() > max_length {
                    &buf[..max_length]
                } else {
                    &buf[..]
                };
                let length = buf.len();

                storage.extend_from_slice(buf);

                let recv_bytes = recv_bytes + length;
                if recv_bytes == firmware_length {
                    self.state = NetConnState::FirmwareWaitO;
                    Ok(length)
                } else {
                    self.state = NetConnState::FirmwareDownload(firmware_length, recv_bytes);
                    Ok(length)
                }
            }
            NetConnState::FirmwareWaitO => {
                if buf[0] == b'O' {
                    self.state = NetConnState::FirmwareWaitK;
                    Ok(1)
                } else {
                    log::error!("End-of-firmware confirmation failed");
                    Err(())
                }
            }
            NetConnState::FirmwareWaitK => {
                if buf[0] == b'K' {
                    log::info!("Firmware successfully downloaded");
                    self.state = NetConnState::WaitCommand;
                    self.firmware_downloaded = true;
                    {
                        let dest = unsafe {
                            core::slice::from_raw_parts_mut(runtime_start, storage.len())
                        };
                        dest.copy_from_slice(storage);
                    }
                    Ok(1)
                } else {
                    log::error!("End-of-firmware confirmation failed");
                    Err(())
                }
            }

            NetConnState::GatewareLength(gateware_length, recv_bytes) => {
                let gateware_length = (gateware_length << 8) | (buf[0] as usize);
                let recv_bytes = recv_bytes + 1;
                if recv_bytes == 4 {
                    self.state = NetConnState::GatewareDownload(gateware_length, 0);
                    storage.clear();
                    storage.reserve_exact(gateware_length);
                } else {
                    self.state = NetConnState::GatewareLength(gateware_length, recv_bytes);
                }
                Ok(1)
            }
            NetConnState::GatewareDownload(gateware_length, recv_bytes) => {
                let max_length = gateware_length - recv_bytes;
                let buf = if buf.len() > max_length {
                    &buf[..max_length]
                } else {
                    &buf[..]
                };
                let length = buf.len();

                storage.extend_from_slice(buf);

                let recv_bytes = recv_bytes + length;
                if recv_bytes == gateware_length {
                    self.state = NetConnState::GatewareWaitO;
                    Ok(length)
                } else {
                    self.state = NetConnState::GatewareDownload(gateware_length, recv_bytes);
                    Ok(length)
                }
            }
            NetConnState::GatewareWaitO => {
                if buf[0] == b'O' {
                    self.state = NetConnState::GatewareWaitK;
                    Ok(1)
                } else {
                    log::error!("End-of-gateware confirmation failed");
                    Err(())
                }
            }
            NetConnState::GatewareWaitK => {
                if buf[0] == b'K' {
                    log::info!("Preprocessing bitstream...");
                    // find sync word 0xFFFFFFFF AA995566
                    let sync_word: [u8; 8] = [0xFF, 0xFF, 0xFF, 0xFF, 0xAA, 0x99, 0x55, 0x66];
                    let mut i = 0;
                    let mut state = 0;
                    while i < storage.len() {
                        if storage[i] == sync_word[state] {
                            state += 1;
                            if state == sync_word.len() {
                                break;
                            }
                        } else {
                            // backtrack
                            // not very efficient but we only have 8 elements
                            'outer: while state > 0 {
                                state -= 1;
                                for j in 0..state {
                                    if storage[i - j] != sync_word[state - j] {
                                        continue 'outer;
                                    }
                                }
                                break;
                            }
                        }
                        i += 1;
                    }
                    if state != sync_word.len() {
                        log::error!("Sync word not found in bitstream (corrupted?)");
                        return Err(());
                    }
                    // we need the sync word
                    // i was pointing to the last element in the sync sequence
                    i -= sync_word.len() - 1;
                    // // append no-op
                    // storage.extend_from_slice(&[0x20, 0, 0, 0]);
                    let bitstream = &mut storage[i..];
                    {
                        // swap endian
                        let swap = unsafe {
                            core::slice::from_raw_parts_mut(
                                bitstream.as_mut_ptr() as usize as *mut u32,
                                bitstream.len() / 4,
                            )
                        };
                        NetworkEndian::from_slice_u32(swap);
                    }
                    unsafe {
                        // align to 64 bytes
                        let ptr = alloc::alloc::alloc(
                            alloc::alloc::Layout::from_size_align(bitstream.len(), 64).unwrap(),
                        );
                        let buffer = core::slice::from_raw_parts_mut(ptr, bitstream.len());
                        buffer.copy_from_slice(bitstream);

                        let mut devcfg = devc::DevC::new();
                        devcfg.enable();
                        let result = devcfg.program(&buffer);
                        core::ptr::drop_in_place(ptr);
                        if let Err(e) = result {
                            log::error!("Error during FPGA startup: {}", e);
                            return Err(());
                        }
                    }

                    log::info!("Gateware successfully downloaded");
                    self.state = NetConnState::WaitCommand;
                    self.gateware_downloaded = true;
                    Ok(1)
                } else {
                    log::info!("End-of-gateware confirmation failed");
                    Err(())
                }
            }
        }
    }

    fn input<File: Read + Seek>(
        &mut self,
        bootgen_file: &mut Option<File>,
        runtime_start: *mut u8,
        runtime_max_len: usize,
        buf: &[u8],
        storage: &mut Vec<u8>,
        mut boot_callback: impl FnMut(),
    ) -> Result<(), ()> {
        let mut remaining = &buf[..];
        while !remaining.is_empty() {
            let read_cnt = self.input_partial(
                bootgen_file,
                runtime_start,
                runtime_max_len,
                remaining,
                storage,
                &mut boot_callback,
            )?;
            remaining = &remaining[read_cnt..];
        }
        Ok(())
    }
}

pub fn netboot<File: Read + Seek>(
    bootgen_file: &mut Option<File>,
    cfg: Config,
    runtime_start: *mut u8,
    runtime_max_len: usize,
) {
    log::info!("Preparing network for netboot");
    let net_addresses = net_settings::get_adresses(&cfg);
    log::info!("Network addresses: {}", net_addresses);
    let eth = Eth::eth0(net_addresses.hardware_addr.0.clone());
    let eth = eth.start_rx(8);
    let mut eth = eth.start_tx(8);

    let mut neighbor_map = [None; 2];
    let neighbor_cache = NeighborCache::new(&mut neighbor_map[..]);
    let mut ip_addrs = [IpCidr::new(net_addresses.ipv4_addr, 0)];
    let mut interface = EthernetInterfaceBuilder::new(&mut eth)
        .ethernet_addr(net_addresses.hardware_addr)
        .ip_addrs(&mut ip_addrs[..])
        .neighbor_cache(neighbor_cache)
        .finalize();

    let mut rx_storage = vec![0; 4096];
    let mut tx_storage = vec![0; 128];

    let mut socket_set_entries: [_; 1] = Default::default();
    let mut sockets = smoltcp::socket::SocketSet::new(&mut socket_set_entries[..]);

    let tcp_rx_buffer = smoltcp::socket::TcpSocketBuffer::new(&mut rx_storage[..]);
    let tcp_tx_buffer = smoltcp::socket::TcpSocketBuffer::new(&mut tx_storage[..]);
    let tcp_socket = smoltcp::socket::TcpSocket::new(tcp_rx_buffer, tcp_tx_buffer);
    let tcp_handle = sockets.add(tcp_socket);

    let mut net_conn = NetConn::new();
    let mut storage = Vec::new();
    let mut boot_flag = false;
    let timer = unsafe { GlobalTimer::get() };

    log::info!("Waiting for connections...");
    loop {
        let timestamp = Instant::from_millis(timer.get_time().0 as i64);
        {
            let socket = &mut *sockets.get::<smoltcp::socket::TcpSocket>(tcp_handle);

            if boot_flag {
                return;
            }
            if !socket.is_open() {
                socket.listen(4269).unwrap() // 0x10ad
            }

            if socket.may_recv() {
                if socket
                    .recv(|data| {
                        (
                            data.len(),
                            net_conn
                                .input(
                                    bootgen_file,
                                    runtime_start,
                                    runtime_max_len,
                                    data,
                                    &mut storage,
                                    || {
                                        boot_flag = true;
                                    },
                                )
                                .is_err(),
                        )
                    })
                    .unwrap()
                {
                    net_conn.reset();
                    socket.close();
                }
            } else if socket.may_send() {
                net_conn.reset();
                socket.close();
            }
        }

        match interface.poll(&mut sockets, timestamp) {
            Ok(_) => (),
            Err(smoltcp::Error::Unrecognized) => (),
            Err(err) => log::error!("Network error: {}", err),
        }
    }
}
