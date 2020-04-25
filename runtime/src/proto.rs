use core::task::Poll;
use core::cmp::min;

use libboard_zynq::smoltcp;
use libasync::smoltcp::TcpStream;


pub type Result<T> = core::result::Result<T, smoltcp::Error>;

pub async fn expect(stream: &TcpStream, pattern: &[u8]) -> Result<bool> {
    stream.recv(|buf| {
        for (i, b) in buf.iter().enumerate() {
            if *b == pattern[i] {
                if i + 1 == pattern.len() {
                    return Poll::Ready((i + 1, Ok(true)));
                }
            } else {
                return Poll::Ready((i + 1, Ok(false)));
            }
        }
        Poll::Pending
    }).await?
}

pub async fn read_bool(stream: &TcpStream) -> Result<bool> {
    Ok(stream.recv(|buf| {
        Poll::Ready((1, buf[0] != 0))
    }).await?)
}

pub async fn read_i8(stream: &TcpStream) -> Result<i8> {
    Ok(stream.recv(|buf| {
        Poll::Ready((1, buf[0] as i8))
    }).await?)
}

pub async fn read_i32(stream: &TcpStream) -> Result<i32> {
    Ok(stream.recv(|buf| {
        if buf.len() >= 4 {
            let value =
                  ((buf[0] as i32) << 24)
                | ((buf[1] as i32) << 16)
                | ((buf[2] as i32) << 8)
                |  (buf[3] as i32);
            Poll::Ready((4, value))
        } else {
            Poll::Pending
        }
    }).await?)
}

pub async fn read_drain(stream: &TcpStream, total: usize) -> Result<()> {
    let mut done = 0;
    while done < total {
        let count = stream.recv(|buf| {
            let count = min(total - done, buf.len());
            Poll::Ready((count, count))
        }).await?;
        done += count;
    }
    Ok(())
}

pub async fn write_i8(stream: &TcpStream, value: i8) -> Result<()> {
    stream.send([value as u8].iter().copied()).await?;
    Ok(())
}

pub async fn write_i32(stream: &TcpStream, value: i32) -> Result<()> {
    stream.send([
        (value >> 24) as u8,
        (value >> 16) as u8,
        (value >>  8) as u8,
         value        as u8].iter().copied()).await?;
    Ok(())
}
