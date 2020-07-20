use futures::{future::poll_fn, task::Poll};
use libasync::{smoltcp::TcpStream, task};
use libboard_zynq::smoltcp;
use core::cell::RefCell;
use alloc::rc::Rc;
use log::{self, info, warn, LevelFilter};

use crate::logger::{BufferLogger, LogBufferRef};
use crate::proto_async;
use crate::proto_mgmt::*;


async fn get_logger_buffer_pred<F>(f: F) -> LogBufferRef<'static>
where
    F: Fn(&LogBufferRef) -> bool,
{
    poll_fn(|ctx| {
        let logger = unsafe { BufferLogger::get_logger().as_mut().unwrap() };
        match logger.buffer() {
            Some(buffer) if f(&buffer) => Poll::Ready(buffer),
            _ => {
                ctx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    })
    .await
}

async fn get_logger_buffer() -> LogBufferRef<'static> {
    get_logger_buffer_pred(|_| true).await
}

async fn handle_connection(stream: &mut TcpStream, pull_id: Rc<RefCell<u32>>) -> Result<(), Error> {
    Request::read_magic(stream).await?;

    loop {
        let req = Request::read_from(stream).await;
        if let Err(Error::Io(smoltcp::Error::Illegal)) = req {
            return Ok(());
        }
        match req? {
            Request::GetLog => {
                let mut buffer = get_logger_buffer().await;
                Reply::LogContent(buffer.extract()).write_to(stream).await?;
            }
            Request::ClearLog => {
                let mut buffer = get_logger_buffer().await;
                buffer.clear();
                Reply::Success.write_to(stream).await?;
            }
            Request::PullLog => {
                let id = {
                    let mut guard = pull_id.borrow_mut();
                    *guard += 1;
                    *guard
                };
                loop {
                    let mut buffer = get_logger_buffer_pred(|b| !b.is_empty()).await;
                    let bytes = buffer.extract().as_bytes();
                    if id != *pull_id.borrow() {
                        // another connection attempts to pull the log...
                        // abort this connection...
                        break;
                    }
                    proto_async::write_chunk(stream, bytes).await?;
                    if log::max_level() == LevelFilter::Trace {
                        // Hold exclusive access over the logger until we get positive
                        // acknowledgement; otherwise we get an infinite loop of network
                        // trace messages being transmitted and causing more network
                        // trace messages to be emitted.
                        //
                        // Any messages unrelated to this management socket that arrive
                        // while it is flushed are lost, but such is life.
                        stream.flush().await?;
                    }
                    buffer.clear();
                }
            },
            Request::SetLogFilter(lvl) => {
                info!("Changing log level to {}", lvl);
                log::set_max_level(lvl);
                Reply::Success.write_to(stream).await?;
            }
            Request::SetUartLogFilter(lvl) => {
                info!("Changing UART log level to {}", lvl);
                unsafe {
                    BufferLogger::get_logger()
                        .as_ref()
                        .unwrap()
                        .set_uart_log_level(lvl);
                }
                Reply::Success.write_to(stream).await?;
            }
        }
    }
}

pub fn start() {
    task::spawn(async move {
        let pull_id = Rc::new(RefCell::new(0u32));
        loop {
            let mut stream = TcpStream::accept(1380, 2048, 2048).await.unwrap();
            let pull_id = pull_id.clone();
            task::spawn(async move {
                info!("received connection");
                let _ = handle_connection(&mut stream, pull_id)
                    .await
                    .map_err(|e| warn!("connection terminated: {:?}", e));
                let _ = stream.flush().await;
                let _ = stream.abort().await;
            });
        }
    });
}
