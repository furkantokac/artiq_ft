#![no_std]

extern crate alloc;
extern crate log;

use core::{mem, ptr, fmt, slice, str, convert, ops::Range};
use alloc::string::String;
use log::{info, trace, error};
use elf::*;

pub mod elf;
mod file;
mod image;
use image::{DynamicSection, Image};
mod reloc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arch {
    Arm,
    OpenRisc,
}


#[derive(Debug)]
pub enum Error {
    Parsing(&'static str),
    Lookup(String)
}

impl convert::From<&'static str> for Error {
    fn from(desc: &'static str) -> Error {
        Error::Parsing(desc)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::Parsing(desc) =>
                write!(f, "parse error: {}", desc),
            &Error::Lookup(ref sym) =>
                write!(f, "symbol lookup error: {}", sym),
        }
    }
}

pub struct Library {
    image: Image,
    dyn_range: Range<usize>,
    dyn_section: DynamicSection<'static>,
}

impl Library {
    pub fn lookup(&self, name: &[u8]) -> Option<u32> {
        self.dyn_section.lookup(name)
            .map(|addr| self.image.ptr() as u32 + addr)
    }
}

pub fn load(
    data: &[u8],
    resolve: &dyn Fn(&[u8]) -> Option<Elf32_Word>
) -> Result<Library, Error> {
    // validate ELF file
    let file = file::File::new(data)
        .ok_or("cannot read ELF header")?;
    if file.ehdr.e_type != ET_DYN {
        return Err("not a shared library")?
    }
    let arch = file.arch()
        .ok_or("not for a supported architecture")?;

    // prepare target memory
    let image_size = file.program_headers()
        .filter_map(|phdr| phdr.map(|phdr| phdr.p_vaddr + phdr.p_memsz))
        .max()
        .unwrap_or(0) as usize;
    let image_align = file.program_headers()
        .filter_map(|phdr| phdr.and_then(|phdr| {
            if phdr.p_type == PT_LOAD {
                Some(phdr.p_align)
            } else {
                None
            }
        }))
        .max()
        .unwrap_or(4) as usize;
    // 1 image for all segments
    let mut image = image::Image::new(image_size, image_align)
        .map_err(|_| "cannot allocate target image")?;
    info!("ELF target: {} bytes, align to {:X}, allocated at {:08X}", image_size, image_align, image.ptr() as usize);

    // LOAD
    for phdr in file.program_headers() {
        let phdr = phdr.ok_or("cannot read program header")?;
        if phdr.p_type != PT_LOAD { continue; }

        trace!("Program header: {:08X}+{:08X} to {:08X}",
              phdr.p_offset, phdr.p_filesz,
              image.ptr() as u32
        );
        let src = file.get(phdr.p_offset as usize..(phdr.p_offset + phdr.p_filesz) as usize)
            .ok_or("program header requests an out of bounds load (in file)")?;
        let dst = image.get_mut(phdr.p_vaddr as usize..
                                (phdr.p_vaddr + phdr.p_filesz) as usize)
            .ok_or("program header requests an out of bounds load (in target)")?;
        dst.copy_from_slice(src);
    }

    // relocate DYNAMIC
    let dyn_range = file.dyn_header_vaddr()
        .ok_or("cannot find a dynamic header")?;
    let dyn_section = image.dyn_section(dyn_range.clone())?;
    info!("Relocating {} rela, {} rel, {} pltrel",
          dyn_section.rela.len(), dyn_section.rel.len(), dyn_section.pltrel.len());

    for rela in dyn_section.rela {
        reloc::relocate(arch, &image, &dyn_section, rela, resolve)?;
    }
    for rel in dyn_section.rela {
        reloc::relocate(arch, &image, &dyn_section, rel, resolve)?;
    }
    for pltrel in dyn_section.pltrel {
        reloc::relocate(arch, &image, &dyn_section, pltrel, resolve)?;
    }

    let dyn_section = unsafe {
        core::mem::transmute(dyn_section)
    };
    Ok(Library {
        image,
        dyn_range,
        dyn_section,
    })
}
