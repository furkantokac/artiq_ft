#![no_std]
#![feature(never_type)]
#![cfg_attr(feature = "alloc", feature(alloc))]

extern crate alloc;
extern crate core_io;

#[cfg(feature = "alloc")]
#[macro_use]
use alloc;
#[cfg(feature = "byteorder")]
extern crate byteorder;

pub mod cursor;
#[cfg(feature = "byteorder")]
pub mod proto;

pub use cursor::Cursor;
#[cfg(feature = "byteorder")]
pub use proto::{ProtoRead, ProtoWrite};
#[cfg(all(feature = "byteorder", feature = "alloc"))]
pub use proto::ReadStringError;