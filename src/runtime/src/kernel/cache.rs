use alloc::string::String;
use cslice::{CSlice, AsCSlice};
use core::mem::transmute;
use super::{KERNEL_CHANNEL_0TO1, KERNEL_CHANNEL_1TO0, Message};

pub extern fn get(key: CSlice<u8>) -> CSlice<'static, i32> {
    let key = String::from_utf8(key.as_ref().to_vec()).unwrap();
    KERNEL_CHANNEL_1TO0.lock().as_mut().unwrap().send(Message::CacheGetRequest(key));
    let msg = KERNEL_CHANNEL_0TO1.lock().as_mut().unwrap().recv();
    if let Message::CacheGetReply(v) = msg {
        let slice = v.as_c_slice();
        // we intentionally leak the memory here,
        // which does not matter as core1 would restart
        unsafe {
            transmute(slice)
        }
    } else {
        panic!("Expected CacheGetReply for CacheGetRequest");
    }
}

pub extern fn put(key: CSlice<u8>, list: CSlice<i32>) {
    let key = String::from_utf8(key.as_ref().to_vec()).unwrap();
    let value = list.as_ref().to_vec();
    KERNEL_CHANNEL_1TO0.lock().as_mut().unwrap().send(Message::CachePutRequest(key, value));
}

