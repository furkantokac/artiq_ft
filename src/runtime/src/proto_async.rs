use core::cmp::min;
use core::cell::RefCell;

use libboard_zynq::smoltcp;
use libasync::smoltcp::TcpStream;

type Result<T> = core::result::Result<T, smoltcp::Error>;

enum RecvState<T> {
    NeedsMore(usize, T), // bytes consumed so far, partial result
    Completed(T),        // final result
}

pub async fn expect(stream: &TcpStream, pattern: &[u8]) -> Result<bool> {
    let mut state = RecvState::NeedsMore(0, true);
    loop {
        state = stream.recv(|buf| {
            let mut consumed = 0;
            if let RecvState::NeedsMore(mut cur_index, _) = state {
                for b in buf.iter() {
                    consumed += 1;
                    if *b == pattern[cur_index] {
                        if cur_index + 1 == pattern.len() {
                            return (consumed, RecvState::Completed(true));
                        }
                    } else {
                        return (consumed, RecvState::Completed(false));
                    }
                    cur_index += 1;
                }
                (consumed, RecvState::NeedsMore(cur_index, true))
            } else {
                unreachable!();
            }
        }).await?;
        if let RecvState::Completed(result) = state {
            return Ok(result);
        }
    }
}

pub async fn read_bool(stream: &TcpStream) -> Result<bool> {
    Ok(stream.recv(|buf| {
        (1, buf[0] != 0)
    }).await?)
}

pub async fn read_i8(stream: &TcpStream) -> Result<i8> {
    Ok(stream.recv(|buf| {
        (1, buf[0] as i8)
    }).await?)
}

pub async fn read_i32(stream: &TcpStream) -> Result<i32> {
    let mut buffer: [u8; 4] = [0; 4];
    read_chunk(stream, &mut buffer).await?;
    Ok(i32::from_be_bytes(buffer))
}

pub async fn read_i64(stream: &TcpStream) -> Result<i64> {
    let mut buffer: [u8; 8] = [0; 8];
    read_chunk(stream, &mut buffer).await?;
    Ok(i64::from_be_bytes(buffer))
}

pub async fn read_chunk(stream: &TcpStream, destination: &mut [u8]) -> Result<()> {
    let total = destination.len();
    let destination = RefCell::new(destination);
    let mut done = 0;
    while done < total {
        let count = stream.recv(|buf| {
            let mut destination = destination.borrow_mut();
            let count = min(total - done, buf.len());
            destination[done..done + count].copy_from_slice(&buf[..count]);
            (count, count)
        }).await?;
        done += count;
    }
    Ok(())
}

pub async fn write_i8(stream: &TcpStream, value: i8) -> Result<()> {
    stream.send_slice(&[value as u8]).await?;
    Ok(())
}

pub async fn write_bool(stream: &TcpStream, value: bool) -> Result<()> {
    stream.send_slice(&[value as u8]).await?;
    Ok(())
}

pub async fn write_i32(stream: &TcpStream, value: i32) -> Result<()> {
    stream.send_slice(&value.to_be_bytes()).await?;
    Ok(())
}

pub async fn write_i64(stream: &TcpStream, value: i64) -> Result<()> {
    stream.send_slice(&value.to_be_bytes()).await?;
    Ok(())
}

pub async fn write_chunk(stream: &TcpStream, value: &[u8]) -> Result<()> {
    write_i32(stream, value.len() as i32).await?;
    stream.send_slice(value).await?;
    Ok(())
}
