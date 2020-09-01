#![no_std]
#![no_main]

extern crate alloc;
extern crate log;

mod netboot;

use alloc::rc::Rc;
use core::mem;
use core_io::{Read, Seek};
use libboard_zynq::{
    self as zynq,
    clocks::source::{ArmPll, ClockSource, IoPll},
    clocks::Clocks,
    logger, println, sdio, slcr,
    timer::GlobalTimer,
};
use libconfig::{bootgen, sd_reader, Config};
use libcortex_a9::{
    asm::{dsb, isb},
    cache::{bpiall, dcciall, iciallu},
    enable_fpu,
    l2c::enable_l2_cache,
};
use libregister::RegisterR;
use libsupport_zynq::ram;
use log::info;

extern "C" {
    static mut __runtime_start: usize;
    static mut __runtime_end: usize;
}

fn boot_sd<File: Read + Seek>(
    file: &mut Option<File>,
    runtime_start: *mut u8,
    runtime_max: usize,
) -> Result<(), ()> {
    if file.is_none() {
        log::error!("No bootgen file");
        return Err(());
    }
    let mut file = file.as_mut().unwrap();
    info!("Loading gateware");
    bootgen::load_bitstream(&mut file).map_err(|e| log::error!("Cannot load gateware: {:?}", e))?;

    info!("Loading runtime");
    let runtime =
        bootgen::get_runtime(&mut file).map_err(|e| log::error!("Cannot load runtime: {:?}", e))?;

    if runtime.len() > runtime_max {
        log::error!(
            "Runtime binary too large, max {} but got {}",
            runtime_max,
            runtime.len()
        );
    }
    unsafe {
        let target = core::slice::from_raw_parts_mut(runtime_start, runtime.len());
        target.copy_from_slice(&runtime);
    }
    Ok(())
}

#[no_mangle]
pub fn main_core0() {
    GlobalTimer::start();
    enable_fpu();
    logger::init().unwrap();
    log::set_max_level(log::LevelFilter::Debug);
    println!(
        r#"

                     __________   __
                    / ___/__  /  / /
                    \__ \  / /  / /
                   ___/ / / /__/ /___
                  /____/ /____/_____/

                   (C) 2020 M-Labs
"#
    );
    info!("Simple Zynq Loader starting...");
    enable_l2_cache();

    const CPU_FREQ: u32 = 800_000_000;

    ArmPll::setup(2 * CPU_FREQ);
    Clocks::set_cpu_freq(CPU_FREQ);
    IoPll::setup(1_000_000_000);
    libboard_zynq::stdio::drop_uart(); // reinitialize UART after clocking change
    let mut ddr = zynq::ddr::DdrRam::ddrram();
    ram::init_alloc_core0();

    let sdio0 = sdio::Sdio::sdio0(true);
    let fs = if sdio0.is_card_inserted() {
        info!("Card inserted. Mounting file system.");
        let sd = sdio::sd_card::SdCard::from_sdio(sdio0).unwrap();
        let reader = sd_reader::SdReader::new(sd);
        reader
            .mount_fatfs(sd_reader::PartitionEntry::Entry1)
            .map(|v| Rc::new(v))
            .ok()
    } else {
        info!("No SD card inserted.");
        None
    };
    let fs_ref = fs.as_ref();
    let root_dir = fs_ref.map(|fs| fs.root_dir());
    let mut bootgen_file = root_dir.and_then(|root_dir| root_dir.open_file("/BOOT.BIN").ok());
    let config = Config::from_fs(fs.clone());

    unsafe {
        let max_len =
            &__runtime_end as *const usize as usize - &__runtime_start as *const usize as usize;
        match slcr::RegisterBlock::unlocked(|slcr| slcr.boot_mode.read().boot_mode_pins()) {
            slcr::BootModePins::Jtag => netboot::netboot(
                &mut bootgen_file,
                config,
                &mut __runtime_start as *mut usize as *mut u8,
                max_len,
            ),
            slcr::BootModePins::SdCard => {
                if boot_sd(
                    &mut bootgen_file,
                    &mut __runtime_start as *mut usize as *mut u8,
                    max_len,
                )
                .is_err()
                {
                    log::error!("Error booting from SD card");
                    log::info!("Fall back on netboot");
                    netboot::netboot(
                        &mut bootgen_file,
                        config,
                        &mut __runtime_start as *mut usize as *mut u8,
                        max_len,
                    )
                }
            }
            v => {
                panic!("Boot mode {:?} not supported", v);
            }
        };
    }

    info!("Preparing for runtime execution");
    // Flush data cache entries for all of L1 cache, including
    // Memory/Instruction Synchronization Barriers
    dcciall();
    iciallu();
    bpiall();
    dsb();
    isb();

    // Start core0 only, for compatibility with FSBL.
    info!("executing payload");
    unsafe {
        (mem::transmute::<*mut u8, fn()>(ddr.ptr::<u8>()))();
    }

    loop {}
}

#[no_mangle]
pub fn main_core1() {
    panic!("core1 started but should not have");
}
