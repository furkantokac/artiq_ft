//! Kernel-side RPC API

use core::mem;
use alloc::{vec::Vec, sync::Arc};
use cslice::{CSlice, AsCSlice};

use libcortex_a9::sync_channel;
use crate::eh_artiq;
use crate::rpc::send_args;
use super::{
    KERNEL_CHANNEL_0TO1, KERNEL_CHANNEL_1TO0,
    Message,
};

fn rpc_send_common(is_async: bool, service: u32, tag: &CSlice<u8>, data: *const *const ()) {
    let core1_tx: &mut sync_channel::Sender<Message> = unsafe { mem::transmute(KERNEL_CHANNEL_1TO0) };
    let mut buffer = Vec::<u8>::new();
    send_args(&mut buffer, service, tag.as_ref(), data).expect("RPC encoding failed");
    core1_tx.send(Message::RpcSend { is_async: is_async, data: Arc::new(buffer) });
}

pub extern fn rpc_send(service: u32, tag: &CSlice<u8>, data: *const *const ()) {
    rpc_send_common(false, service, tag, data);
}

pub extern fn rpc_send_async(service: u32, tag: &CSlice<u8>, data: *const *const ()) {
    rpc_send_common(true, service, tag, data);
}

pub extern fn rpc_recv(slot: *mut ()) -> usize {
    let core1_rx: &mut sync_channel::Receiver<Message> = unsafe { mem::transmute(KERNEL_CHANNEL_0TO1) };
    let core1_tx: &mut sync_channel::Sender<Message> = unsafe { mem::transmute(KERNEL_CHANNEL_1TO0) };
    core1_tx.send(Message::RpcRecvRequest(slot));
    let reply = core1_rx.recv();
    match *reply {
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
