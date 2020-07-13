use core::cell::{Cell, RefCell, RefMut};
use core::fmt::Write;
use log::{Log, LevelFilter};
use log_buffer::LogBuffer;
use libboard_zynq::{println, timer::GlobalTimer};

pub struct LogBufferRef<'a> {
    buffer:        RefMut<'a, LogBuffer<&'static mut [u8]>>,
    old_log_level: LevelFilter
}

impl<'a> LogBufferRef<'a> {
    fn new(buffer: RefMut<'a, LogBuffer<&'static mut [u8]>>) -> LogBufferRef<'a> {
        let old_log_level = log::max_level();
        log::set_max_level(LevelFilter::Off);
        LogBufferRef { buffer, old_log_level }
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn clear(&mut self) {
        self.buffer.clear()
    }

    pub fn extract(&mut self) -> &str {
        self.buffer.extract()
    }
}

impl<'a> Drop for LogBufferRef<'a> {
    fn drop(&mut self) {
        log::set_max_level(self.old_log_level)
    }
}

pub struct BufferLogger {
    buffer:      RefCell<LogBuffer<&'static mut [u8]>>,
    uart_filter: Cell<LevelFilter>
}

static mut LOGGER: Option<BufferLogger> = None;

impl BufferLogger {
    pub fn new(buffer: &'static mut [u8]) -> BufferLogger {
        BufferLogger {
            buffer: RefCell::new(LogBuffer::new(buffer)),
            uart_filter: Cell::new(LevelFilter::Info),
        }
    }

    pub fn register(self) {
        unsafe {
            LOGGER = Some(self);
            log::set_logger(LOGGER.as_ref().unwrap())
                .expect("global logger can only be initialized once");
        }
    }

    pub unsafe fn get_logger() -> &'static mut Option<BufferLogger> {
        &mut LOGGER
    }

    pub fn buffer<'a>(&'a self) -> Result<LogBufferRef<'a>, ()> {
        self.buffer
            .try_borrow_mut()
            .map(LogBufferRef::new)
            .map_err(|_| ())
    }

    pub fn uart_log_level(&self) -> LevelFilter {
        self.uart_filter.get()
    }

    pub fn set_uart_log_level(&self, max_level: LevelFilter) {
        self.uart_filter.set(max_level)
    }
}

// required for impl Log
unsafe impl Sync for BufferLogger {}

impl Log for BufferLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let timestamp = unsafe {
                GlobalTimer::get()
            }.get_us();
            let seconds   = timestamp / 1_000_000;
            let micros    = timestamp % 1_000_000;

            if let Ok(mut buffer) = self.buffer.try_borrow_mut() {
                writeln!(buffer, "[{:6}.{:06}s] {:>5}({}): {}", seconds, micros,
                         record.level(), record.target(), record.args()).unwrap();
            }

            if record.level() <= self.uart_log_level() {
                println!("[{:6}.{:06}s] {:>5}({}): {}", seconds, micros,
                         record.level(), record.target(), record.args());
            }
        }
    }

    fn flush(&self) {
    }
}
