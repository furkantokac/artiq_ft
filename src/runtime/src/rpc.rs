use core::str;
use cslice::{CSlice, CMutSlice};
use log::debug;

use libasync::smoltcp::TcpStream;

use crate::proto::*;
use self::tag::{Tag, TagIterator, split_tag};

/* TODO: figure out Rust problems with recursive async fns */

async unsafe fn recv_value(stream: &TcpStream, tag: Tag<'_>, data: &mut *mut (),
                     alloc: &dyn Fn(usize) -> *mut ())
                    -> Result<()>
{
    macro_rules! consume_value {
        ($ty:ty, |$ptr:ident| $map:expr) => ({
            let $ptr = (*data) as *mut $ty;
            *data = $ptr.offset(1) as *mut ();
            $map
        })
    }

    match tag {
        Tag::None => Ok(()),
        Tag::Bool =>
            consume_value!(i8, |ptr| {
                *ptr = read_i8(stream).await?;
                Ok(())
            }),
        Tag::Int32 =>
            consume_value!(i32, |ptr| {
                *ptr = read_i32(stream).await?;
                Ok(())
            }),
        Tag::Int64 | Tag::Float64 =>
            consume_value!(i64, |ptr| {
                *ptr = read_i64(stream).await?;
                Ok(())
            }),
        Tag::String | Tag::Bytes | Tag::ByteArray => {
            consume_value!(CMutSlice<u8>, |ptr| {
                let length = read_i32(stream).await? as usize;
                *ptr = CMutSlice::new(alloc(length) as *mut u8, length);
                read_chunk(stream, (*ptr).as_mut()).await?;
                Ok(())
            })
        }
        Tag::Tuple(it, arity) => {
            let mut it = it.clone();
            for _ in 0..arity {
                let tag = it.next().expect("truncated tag");
                // TODO recv_value(stream, tag, data, alloc).await?
            }
            Ok(())
        }
        Tag::List(it) | Tag::Array(it) => {
            struct List { elements: *mut (), length: u32 };
            consume_value!(List, |ptr| {
                (*ptr).length = read_i32(stream).await? as u32;

                let tag = it.clone().next().expect("truncated tag");
                (*ptr).elements = alloc(tag.size() * (*ptr).length as usize);

                let mut data = (*ptr).elements;
                for _ in 0..(*ptr).length as usize {
                    // TODO recv_value(stream, tag, &mut data, alloc).await?
                }
                Ok(())
            })
        }
        Tag::Range(it) => {
            let tag = it.clone().next().expect("truncated tag");
            // TODO recv_value(stream, tag, data, alloc).await?;
            // TODO recv_value(stream, tag, data, alloc).await?;
            // TODO recv_value(stream, tag, data, alloc).await?;
            Ok(())
        }
        Tag::Keyword(_) => unreachable!(),
        Tag::Object => unreachable!()
    }
}

pub async fn recv_return(stream: &TcpStream, tag_bytes: &[u8], data: *mut (),
                         alloc: &dyn Fn(usize) -> *mut ())
                        -> Result<()>
{
    let mut it = TagIterator::new(tag_bytes);
    debug!("recv ...->{}", it);

    let tag = it.next().expect("truncated tag");
    let mut data = data;
    unsafe { recv_value(stream, tag, &mut data, alloc).await? };

    Ok(())
}

async unsafe fn send_value(stream: &TcpStream, tag: Tag<'_>, data: &mut *const ())
                           -> Result<()>
{
    macro_rules! consume_value {
        ($ty:ty, |$ptr:ident| $map:expr) => ({
            let $ptr = (*data) as *const $ty;
            *data = $ptr.offset(1) as *const ();
            $map
        })
    }

    write_i8(stream, tag.as_u8() as i8).await?;
    match tag {
        Tag::None => Ok(()),
        Tag::Bool =>
            consume_value!(i8, |ptr|
                write_i8(stream, *ptr).await),
        Tag::Int32 =>
            consume_value!(i32, |ptr|
                write_i32(stream, *ptr).await),
        Tag::Int64 | Tag::Float64 =>
            consume_value!(i64, |ptr|
                write_i64(stream, *ptr).await),
        Tag::String =>
            //consume_value!(CSlice<u8>, |ptr|
            //    writer.write_string(str::from_utf8((*ptr).as_ref()).unwrap())),
            // TODO
            unimplemented!(),
        Tag::Bytes | Tag::ByteArray =>
            consume_value!(CSlice<u8>, |ptr|
                stream.send((*ptr).as_ref().iter().copied()).await),
        Tag::Tuple(it, arity) => {
            let mut it = it.clone();
            write_i8(stream, arity as i8).await?;
            for _ in 0..arity {
                let tag = it.next().expect("truncated tag");
                // TODO send_value(stream, tag, data).await?
            }
            Ok(())
        }
        Tag::List(it) | Tag::Array(it) => {
            struct List { elements: *const (), length: u32 };
            consume_value!(List, |ptr| {
                write_i32(stream, (*ptr).length as i32).await?;
                let tag = it.clone().next().expect("truncated tag");
                let mut data = (*ptr).elements;
                for _ in 0..(*ptr).length as usize {
                    // TODO send_value(stream, tag, &mut data).await?;
                }
                Ok(())
            })
        }
        Tag::Range(it) => {
            let tag = it.clone().next().expect("truncated tag");
            // TODO send_value(stream, tag, data).await?;
            // TODO send_value(stream, tag, data).await?;
            // TODO send_value(stream, tag, data).await?;
            Ok(())
        }
        Tag::Keyword(it) => {
            struct Keyword<'a> { name: CSlice<'a, u8> };
            consume_value!(Keyword, |ptr| {
                //TODO writer.write_string(str::from_utf8((*ptr).name.as_ref()).unwrap())?;
                let tag = it.clone().next().expect("truncated tag");
                let mut data = ptr.offset(1) as *const ();
                // TODO send_value(stream, tag, &mut data).await
                Ok(())
            })
            // Tag::Keyword never appears in composite types, so we don't have
            // to accurately advance data.
        }
        Tag::Object => {
            struct Object { id: u32 };
            consume_value!(*const Object, |ptr|
                write_i32(stream, (**ptr).id as i32).await)
        }
    }
}

pub async fn send_args(stream: &TcpStream, service: u32, tag_bytes: &[u8], data: *const *const ())
                      -> Result<()>
{
    let (arg_tags_bytes, return_tag_bytes) = split_tag(tag_bytes);

    let mut args_it = TagIterator::new(arg_tags_bytes);
    let return_it = TagIterator::new(return_tag_bytes);
    debug!("send<{}>({})->{}", service, args_it, return_it);

    write_i32(stream, service as i32).await?;
    for index in 0.. {
        if let Some(arg_tag) = args_it.next() {
            let mut data = unsafe { *data.offset(index) };
            unsafe { send_value(stream, arg_tag, &mut data).await? };
        } else {
            break
        }
    }
    write_i8(stream, 0).await?;
    stream.send(return_tag_bytes.iter().copied()).await?;

    Ok(())
}

mod tag {
    use core::fmt;

    pub fn split_tag(tag_bytes: &[u8]) -> (&[u8], &[u8]) {
        let tag_separator =
            tag_bytes.iter()
                     .position(|&b| b == b':')
                     .expect("tag without a return separator");
        let (arg_tags_bytes, rest) = tag_bytes.split_at(tag_separator);
        let return_tag_bytes = &rest[1..];
        (arg_tags_bytes, return_tag_bytes)
    }

    #[derive(Debug, Clone, Copy)]
    pub enum Tag<'a> {
        None,
        Bool,
        Int32,
        Int64,
        Float64,
        String,
        Bytes,
        ByteArray,
        Tuple(TagIterator<'a>, u8),
        List(TagIterator<'a>),
        Array(TagIterator<'a>),
        Range(TagIterator<'a>),
        Keyword(TagIterator<'a>),
        Object
    }

    impl<'a> Tag<'a> {
        pub fn as_u8(self) -> u8 {
            match self {
                Tag::None => b'n',
                Tag::Bool => b'b',
                Tag::Int32 => b'i',
                Tag::Int64 => b'I',
                Tag::Float64 => b'f',
                Tag::String => b's',
                Tag::Bytes => b'B',
                Tag::ByteArray => b'A',
                Tag::Tuple(_, _) => b't',
                Tag::List(_) => b'l',
                Tag::Array(_) => b'a',
                Tag::Range(_) => b'r',
                Tag::Keyword(_) => b'k',
                Tag::Object => b'O',
            }
        }

        pub fn size(self) -> usize {
            match self {
                Tag::None => 0,
                Tag::Bool => 1,
                Tag::Int32 => 4,
                Tag::Int64 => 8,
                Tag::Float64 => 8,
                Tag::String => 8,
                Tag::Bytes => 8,
                Tag::ByteArray => 8,
                Tag::Tuple(it, arity) => {
                    let mut size = 0;
                    let mut it = it.clone();
                    for _ in 0..arity {
                        let tag = it.next().expect("truncated tag");
                        size += tag.size();
                    }
                    size
                }
                Tag::List(_) => 8,
                Tag::Array(_) => 8,
                Tag::Range(it) => {
                    let tag = it.clone().next().expect("truncated tag");
                    tag.size() * 3
                }
                Tag::Keyword(_) => unreachable!(),
                Tag::Object => unreachable!(),
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct TagIterator<'a> {
        data: &'a [u8]
    }

    impl<'a> TagIterator<'a> {
        pub fn new(data: &'a [u8]) -> TagIterator<'a> {
            TagIterator { data: data }
        }

        pub fn next(&mut self) -> Option<Tag<'a>> {
            if self.data.len() == 0 {
                return None
            }

            let tag_byte = self.data[0];
            self.data = &self.data[1..];
            Some(match tag_byte {
                b'n' => Tag::None,
                b'b' => Tag::Bool,
                b'i' => Tag::Int32,
                b'I' => Tag::Int64,
                b'f' => Tag::Float64,
                b's' => Tag::String,
                b'B' => Tag::Bytes,
                b'A' => Tag::ByteArray,
                b't' => {
                    let count = self.data[0];
                    self.data = &self.data[1..];
                    Tag::Tuple(self.sub(count), count)
                }
                b'l' => Tag::List(self.sub(1)),
                b'a' => Tag::Array(self.sub(1)),
                b'r' => Tag::Range(self.sub(1)),
                b'k' => Tag::Keyword(self.sub(1)),
                b'O' => Tag::Object,
                _    => unreachable!()
            })
        }

        fn sub(&mut self, count: u8) -> TagIterator<'a> {
            let data = self.data;
            for _ in 0..count {
                self.next().expect("truncated tag");
            }
            TagIterator { data: &data[..(data.len() - self.data.len())] }
        }
    }

    impl<'a> fmt::Display for TagIterator<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let mut it = self.clone();
            let mut first = true;
            while let Some(tag) = it.next() {
                if first {
                    first = false
                } else {
                    write!(f, ", ")?
                }

                match tag {
                    Tag::None =>
                        write!(f, "None")?,
                    Tag::Bool =>
                        write!(f, "Bool")?,
                    Tag::Int32 =>
                        write!(f, "Int32")?,
                    Tag::Int64 =>
                        write!(f, "Int64")?,
                    Tag::Float64 =>
                        write!(f, "Float64")?,
                    Tag::String =>
                        write!(f, "String")?,
                    Tag::Bytes =>
                        write!(f, "Bytes")?,
                    Tag::ByteArray =>
                        write!(f, "ByteArray")?,
                    Tag::Tuple(it, _) => {
                        write!(f, "Tuple(")?;
                        it.fmt(f)?;
                        write!(f, ")")?;
                    }
                    Tag::List(it) => {
                        write!(f, "List(")?;
                        it.fmt(f)?;
                        write!(f, ")")?;
                    }
                    Tag::Array(it) => {
                        write!(f, "Array(")?;
                        it.fmt(f)?;
                        write!(f, ")")?;
                    }
                    Tag::Range(it) => {
                        write!(f, "Range(")?;
                        it.fmt(f)?;
                        write!(f, ")")?;
                    }
                    Tag::Keyword(it) => {
                        write!(f, "Keyword(")?;
                        it.fmt(f)?;
                        write!(f, ")")?;
                    }
                    Tag::Object =>
                        write!(f, "Object")?,
                }
            }

            Ok(())
        }
    }
}
