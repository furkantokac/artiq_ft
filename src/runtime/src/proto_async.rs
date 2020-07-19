use core::task::Poll;
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
                            return Poll::Ready((consumed, RecvState::Completed(true)));
                        }
                    } else {
                        return Poll::Ready((consumed, RecvState::Completed(false)));
                    }
                    cur_index += 1;
                }
                Poll::Ready((consumed, RecvState::NeedsMore(cur_index, true)))
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
        Poll::Ready((1, buf[0] != 0))
    }).await?)
}

pub async fn read_i8(stream: &TcpStream) -> Result<i8> {
    Ok(stream.recv(|buf| {
        Poll::Ready((1, buf[0] as i8))
    }).await?)
}

pub async fn read_i32(stream: &TcpStream) -> Result<i32> {
    let mut state = RecvState::NeedsMore(0, 0);
    loop {
        state = stream.recv(|buf| {
            let mut consumed = 0;
            if let RecvState::NeedsMore(mut cur_index, mut cur_value) = state {
                for b in buf.iter() {
                    consumed += 1;
                    cur_index += 1;
                    cur_value <<= 8;
                    cur_value |= *b as i32;
                    if cur_index == 4 {
                        return Poll::Ready((consumed, RecvState::Completed(cur_value)));
                    }
                }
                Poll::Ready((consumed, RecvState::NeedsMore(cur_index, cur_value)))
            } else {
                unreachable!();
            }
        }).await?;
        if let RecvState::Completed(result) = state {
            return Ok(result);
        }
    }
}

pub async fn read_i64(stream: &TcpStream) -> Result<i64> {
    let mut state = RecvState::NeedsMore(0, 0);
    loop {
        state = stream.recv(|buf| {
            let mut consumed = 0;
            if let RecvState::NeedsMore(mut cur_index, mut cur_value) = state {
                for b in buf.iter() {
                    consumed += 1;
                    cur_index += 1;
                    cur_value <<= 8;
                    cur_value |= *b as i64;
                    if cur_index == 8 {
                        return Poll::Ready((consumed, RecvState::Completed(cur_value)));
                    }
                }
                Poll::Ready((consumed, RecvState::NeedsMore(cur_index, cur_value)))
            } else {
                unreachable!();
            }
        }).await?;
        if let RecvState::Completed(result) = state {
            return Ok(result);
        }
    }
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

pub async fn write_bool(stream: &TcpStream, value: bool) -> Result<()> {
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

pub async fn write_i64(stream: &TcpStream, value: i64) -> Result<()> {
    stream.send([
        (value >> 56) as u8,
        (value >> 48) as u8,
        (value >> 40) as u8,
        (value >> 32) as u8,
        (value >> 24) as u8,
        (value >> 16) as u8,
        (value >>  8) as u8,
         value        as u8].iter().copied()).await?;
    Ok(())
}

pub async fn write_chunk(stream: &TcpStream, value: &[u8]) -> Result<()> {
    write_i32(stream, value.len() as i32).await?;
    stream.send(value.iter().copied()).await?;
    Ok(())
}
