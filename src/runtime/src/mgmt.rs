use futures::{future::poll_fn, task::Poll};
use libasync::{smoltcp::TcpStream, task};
use libboard_zynq::smoltcp;
use log::{self, info, warn};

use crate::logger::{BufferLogger, LogBufferRef};
use crate::proto_async;
use crate::proto_mgmt::*;

async fn get_logger_buffer() -> LogBufferRef<'static> {
    poll_fn(|ctx| {
        let logger = unsafe { BufferLogger::get_logger().as_mut().unwrap() };
        match logger.buffer() {
            Ok(buffer) => Poll::Ready(buffer),
            _ => {
                ctx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    })
    .await
}

async fn handle_connection(stream: &mut TcpStream) -> Result<(), Error> {
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
            Request::PullLog => loop {
                let mut buffer = get_logger_buffer().await;
                if buffer.is_empty() {
                    continue;
                }
                proto_async::write_chunk(stream, buffer.extract().as_bytes()).await?;
                buffer.clear();
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
            },
        }
    }
}

pub fn start() {
    task::spawn(async move {
        loop {
            let mut stream = TcpStream::accept(1380, 2048, 2048).await.unwrap();
            task::spawn(async move {
                let _ = handle_connection(&mut stream)
                    .await
                    .map_err(|e| warn!("connection terminated: {:?}", e));
                let _ = stream.flush().await;
                let _ = stream.abort().await;
            });
        }
    });
}
