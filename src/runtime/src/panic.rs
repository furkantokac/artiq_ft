use libboard_zynq::{print, println};
use unwind::backtrace;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
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
