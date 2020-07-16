use libboard_zynq::{print, println};
use libregister::RegisterR;
use libcortex_a9::regs::MPIDR;
use unwind::backtrace;

static mut PANICKED: [bool; 2] = [false; 2];

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let id = MPIDR.read().cpu_id() as usize;
    print!("Core {} ", id);
    unsafe {
        if PANICKED[id] {
            println!("nested panic!");
            loop {}
        }
        PANICKED[id] = true;
    }
    print!("panic at ");
    if let Some(location) = info.location() {
        print!("{}:{}:{}", location.file(), location.line(), location.column());
    } else {
        print!("unknown location");
    }
    if let Some(message) = info.message() {
        println!(": {}", message);
    } else {
        println!("");
    }
    println!("Backtrace: ");
    let _ = backtrace(|ip| {
        // Backtrace gives us the return address, i.e. the address after the delay slot,
        // but we're interested in the call instruction.
        print!("{:#08x} ", ip - 2 * 4);
    });
    println!("\nEnd backtrace");

    loop {}
}
