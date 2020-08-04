//! Kernel-side RPC API

use alloc::vec::Vec;
use cslice::{CSlice, AsCSlice};

use crate::eh_artiq;
use crate::rpc::send_args;
use super::{
    KERNEL_CHANNEL_0TO1, KERNEL_CHANNEL_1TO0,
    Message,
};

fn rpc_send_common(is_async: bool, service: u32, tag: &CSlice<u8>, data: *const *const ()) {
    let mut core1_tx = KERNEL_CHANNEL_1TO0.lock();
    let mut buffer = Vec::<u8>::new();
    send_args(&mut buffer, service, tag.as_ref(), data).expect("RPC encoding failed");
    core1_tx.as_mut().unwrap().send(Message::RpcSend { is_async, data: buffer });
}

pub extern fn rpc_send(service: u32, tag: &CSlice<u8>, data: *const *const ()) {
    rpc_send_common(false, service, tag, data);
}

pub extern fn rpc_send_async(service: u32, tag: &CSlice<u8>, data: *const *const ()) {
    rpc_send_common(true, service, tag, data);
}

pub extern fn rpc_recv(slot: *mut ()) -> usize {
    let reply = {
        let mut core1_rx = KERNEL_CHANNEL_0TO1.lock();
        let mut core1_tx = KERNEL_CHANNEL_1TO0.lock();
        core1_tx.as_mut().unwrap().send(Message::RpcRecvRequest(slot));
        core1_rx.as_mut().unwrap().recv()
    };
    match reply {
        Message::RpcRecvReply(Ok(alloc_size)) => alloc_size,
        Message::RpcRecvReply(Err(exception)) => unsafe {
            eh_artiq::raise(&eh_artiq::Exception {
                name:     exception.name.as_bytes().as_c_slice(),
                file:     exception.file.as_bytes().as_c_slice(),
                line:     exception.line as u32,
                column:   exception.column as u32,
                function: exception.function.as_bytes().as_c_slice(),
                message:  exception.message.as_bytes().as_c_slice(),
                param:    exception.param
            })
        },
        _ => panic!("received unexpected reply to RpcRecvRequest: {:?}", reply)
    }
}
