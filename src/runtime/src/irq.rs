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
    asm!(
        // setup SP, depending on CPU 0 or 1
        "mrc p15, #0, r0, c0, c0, #5",
        "movw r1, :lower16:__stack0_start",
        "movt r1, :upper16:__stack0_start",
        "tst r0, #3",
        "movwne r1, :lower16:__stack1_start",
        "movtne r1, :upper16:__stack1_start",
        "mov sp, r1",
        "bl __IRQ",
        options(noreturn)
    );
}

#[no_mangle]
pub unsafe extern "C" fn __IRQ() {
    if MPIDR.read().cpu_id() == 1 {
        let mpcore = mpcore::RegisterBlock::mpcore();
        let mut gic = gic::InterruptController::gic(mpcore);
        let id = gic.get_interrupt_id();
        if id.0 == 0 {
            gic.end_interrupt(id);
            // save the SP and set it back after exiting IRQ
            // exception unwinding expect to unwind from this function, as this is not the entrance
            // function, maybe to IRQ which cannot further unwind...
            // if we set the SP to __stack1_start, interesting exceptions would be triggered when
            // we try to unwind the stack...
            let v = SP.read();
            asm::exit_irq();
            SP.write(v);
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
    let mut interrupt_controller = gic::InterruptController::gic(mpcore::RegisterBlock::mpcore());
    CORE1_RESTART.store(true, Ordering::Relaxed);
    interrupt_controller.send_sgi(gic::InterruptId(0), gic::CPUCore::Core1.into());
    while CORE1_RESTART.load(Ordering::Relaxed) {
        spin_lock_yield();
    }
}
