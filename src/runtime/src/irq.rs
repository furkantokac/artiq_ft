use libboard_zynq::{gic, mpcore, println, stdio};
use libcortex_a9::{
    asm,
    regs::{MPIDR, SP},
    spin_lock_yield, notify_spin_lock
};
use libregister::{RegisterR, RegisterW};
use core::sync::atomic::{AtomicBool, Ordering};

extern "C" {
    static mut __stack1_start: u32;
    fn main_core1() -> !;
}

static CORE1_RESTART: AtomicBool = AtomicBool::new(false);

#[link_section = ".text.boot"]
#[no_mangle]
#[naked]
pub unsafe extern "C" fn IRQ() {
    if MPIDR.read().cpu_id() == 1 {
        let mpcore = mpcore::RegisterBlock::new();
        let mut gic = gic::InterruptController::new(mpcore);
        let id = gic.get_interrupt_id();
        if id.0 == 0 {
            gic.end_interrupt(id);
            asm::exit_irq();
            SP.write(&mut __stack1_start as *mut _ as u32);
            asm::enable_irq();
            CORE1_RESTART.store(false, Ordering::Relaxed);
            notify_spin_lock();
            main_core1();
        }
    }
    stdio::drop_uart();
    println!("IRQ");
    loop {}
}

pub fn restart_core1() {
    let mut interrupt_controller = gic::InterruptController::new(mpcore::RegisterBlock::new());
    CORE1_RESTART.store(true, Ordering::Relaxed);
    interrupt_controller.send_sgi(gic::InterruptId(0), gic::CPUCore::Core1.into());
    while CORE1_RESTART.load(Ordering::Relaxed) {
        spin_lock_yield();
    }
}
