use alloc::{string::String, vec::Vec, collections::BTreeMap};
use libcortex_a9::mutex::Mutex;
use cslice::{CSlice, AsCSlice};
use core::mem::transmute;
use core::str;
use log::debug;

use crate::artiq_raise;


#[derive(Debug)]
struct Entry {
    data: Vec<i32>,
    borrowed: bool
}

#[derive(Debug)]
struct Cache {
    entries: BTreeMap<String, Entry>
}

impl Cache {
    pub const fn new() -> Cache {
        Cache { entries: BTreeMap::new() }
    }

    pub fn get(&mut self, key: &str) -> *const [i32] {
        match self.entries.get_mut(key) {
            None => &[],
            Some(ref mut entry) => {
                entry.borrowed = true;
                &entry.data[..]
            }
        }
    }

    pub fn put(&mut self, key: &str, data: &[i32]) -> Result<(), ()> {
        match self.entries.get_mut(key) {
            None => (),
            Some(ref mut entry) => {
                if entry.borrowed { return Err(()) }
                entry.data = Vec::from(data);
                return Ok(())
            }
        }

        self.entries.insert(String::from(key), Entry {
            data: Vec::from(data),
            borrowed: false
        });
        Ok(())
    }

    pub unsafe fn unborrow(&mut self) {
        for (_key, entry) in self.entries.iter_mut() {
            entry.borrowed = false;
        }
    }
}

static CACHE: Mutex<Cache> = Mutex::new(Cache::new());

pub extern fn get(key: CSlice<u8>) -> CSlice<'static, i32> {
    let value = CACHE.lock().get(str::from_utf8(key.as_ref()).unwrap());
    unsafe {
        transmute::<*const [i32], &'static [i32]>(value).as_c_slice()
    }
}

pub extern fn put(key: CSlice<u8>, list: CSlice<i32>) {
    let result = CACHE.lock().put(str::from_utf8(key.as_ref()).unwrap(), list.as_ref());
    if result.is_err() {
        artiq_raise!("CacheError", "cannot put into a busy cache row");
    }
}

pub unsafe fn unborrow() {
    CACHE.lock().unborrow();
}
