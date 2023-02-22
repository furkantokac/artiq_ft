#![no_std]

extern crate alloc;
extern crate libcortex_a9;
extern crate log;

use alloc::string::String;
use core::{convert, fmt, ops::Range, str};

use elf::*;
use log::{debug, trace};

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
    Lookup(String),
}

impl convert::From<&'static str> for Error {
    fn from(desc: &'static str) -> Error {
        Error::Parsing(desc)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::Parsing(desc) => write!(f, "parse error: {}", desc),
            &Error::Lookup(ref sym) => write!(f, "symbol lookup error: {}", sym),
        }
    }
}

fn elf_hash(name: &[u8]) -> u32 {
    let mut h: u32 = 0;
    for c in name {
        h = (h << 4) + *c as u32;
        let g = h & 0xf0000000;
        if g != 0 {
            h ^= g >> 24;
            h &= !g;
        }
    }
    h
}

pub struct Library {
    pub image: Image,
    pub arch: Arch,
    dyn_section: DynamicSection,
    exidx: Range<usize>,
}

impl Library {
    fn strtab(&self) -> &[u8] {
        self.image.get_ref_slice_unchecked(&self.dyn_section.strtab)
    }

    fn symtab(&self) -> &[Elf32_Sym] {
        self.image.get_ref_slice_unchecked(&self.dyn_section.symtab)
    }

    fn hash(&self) -> &[Elf32_Word] {
        self.image.get_ref_slice_unchecked(&self.dyn_section.hash)
    }

    fn hash_bucket(&self) -> &[Elf32_Word] {
        &self.hash()[self.dyn_section.hash_bucket.clone()]
    }

    fn hash_chain(&self) -> &[Elf32_Word] {
        &self.hash()[self.dyn_section.hash_chain.clone()]
    }

    fn rel(&self) -> &[Elf32_Rel] {
        self.image.get_ref_slice_unchecked(&self.dyn_section.rel)
    }

    fn rela(&self) -> &[Elf32_Rela] {
        self.image.get_ref_slice_unchecked(&self.dyn_section.rela)
    }

    fn pltrel(&self) -> &[Elf32_Rel] {
        self.image.get_ref_slice_unchecked(&self.dyn_section.pltrel)
    }

    pub fn lookup(&self, name: &[u8]) -> Option<Elf32_Word> {
        let hash = elf_hash(name);
        let mut index = self.hash_bucket()[hash as usize % self.hash_bucket().len()] as usize;

        loop {
            if index == STN_UNDEF {
                return None;
            }

            let sym = &self.symtab()[index];
            let sym_name_off = sym.st_name as usize;
            match self.strtab().get(sym_name_off..sym_name_off + name.len()) {
                Some(sym_name) if sym_name == name => {
                    if ELF32_ST_BIND(sym.st_info) & STB_GLOBAL == 0 {
                        return None;
                    }

                    match sym.st_shndx {
                        SHN_UNDEF => return None,
                        SHN_ABS => return Some(self.image.ptr() as u32 + sym.st_value),
                        _ => return Some(self.image.ptr() as u32 + sym.st_value),
                    }
                }
                _ => (),
            }

            index = self.hash_chain()[index] as usize;
        }
    }

    pub fn name_starting_at(&self, offset: usize) -> Result<&[u8], Error> {
        let size = self
            .strtab()
            .iter()
            .skip(offset)
            .position(|&x| x == 0)
            .ok_or("symbol in symbol table not null-terminated")?;
        Ok(self
            .strtab()
            .get(offset..offset + size)
            .ok_or("cannot read symbol name")?)
    }

    /// Rebind Rela by `name` to a new `addr`
    pub fn rebind(&self, name: &[u8], addr: *const ()) -> Result<(), Error> {
        reloc::rebind(self.arch, self, name, addr as Elf32_Word)
    }

    pub fn exidx(&self) -> &[EXIDX_Entry] {
        self.image.get_ref_slice_unchecked(&self.exidx)
    }
}

pub fn load(data: &[u8], resolve: &dyn Fn(&[u8]) -> Option<Elf32_Word>) -> Result<Library, Error> {
    // validate ELF file
    let file = file::File::new(data).ok_or("cannot read ELF header")?;
    if file.ehdr.e_type != ET_DYN {
        return Err("not a shared library")?;
    }
    let arch = file.arch().ok_or("not for a supported architecture")?;

    // prepare target memory
    let image_size = file
        .program_headers()
        .filter_map(|phdr| phdr.map(|phdr| phdr.p_vaddr + phdr.p_memsz))
        .max()
        .unwrap_or(0) as usize;
    let image_align = file
        .program_headers()
        .filter_map(|phdr| {
            phdr.and_then(|phdr| {
                if phdr.p_type == PT_LOAD {
                    Some(phdr.p_align)
                } else {
                    None
                }
            })
        })
        .max()
        .unwrap_or(4) as usize;
    // 1 image for all segments
    let mut image = image::Image::new(image_size, image_align).map_err(|_| "cannot allocate target image")?;
    debug!(
        "ELF target: {} bytes, align to {:X}, allocated at {:08X}",
        image_size,
        image_align,
        image.ptr() as usize
    );

    // LOAD
    for phdr in file.program_headers() {
        let phdr = phdr.ok_or("cannot read program header")?;
        trace!(
            "Program header: {:08X}+{:08X} to {:08X}",
            phdr.p_offset,
            phdr.p_filesz,
            image.ptr() as u32
        );
        let file_range = phdr.p_offset as usize..(phdr.p_offset + phdr.p_filesz) as usize;
        match phdr.p_type {
            PT_LOAD => {
                let src = file
                    .get(file_range)
                    .ok_or("program header requests an out of bounds load (in file)")?;
                let dst = image
                    .get_mut(phdr.p_vaddr as usize..(phdr.p_vaddr + phdr.p_filesz) as usize)
                    .ok_or("program header requests an out of bounds load (in target)")?;
                dst.copy_from_slice(src);
            }
            _ => {}
        }
    }

    let mut exidx = None;
    // Obtain EXIDX
    for shdr in file.section_headers() {
        let shdr = shdr.ok_or("cannot read section header")?;
        match shdr.sh_type as usize {
            SHT_ARM_EXIDX => {
                let range = shdr.sh_addr as usize..(shdr.sh_addr + shdr.sh_size) as usize;
                let _ = image
                    .get(range.clone())
                    .ok_or("section header specifies EXIDX outside of image (in target)")?;
                exidx = Some(range);
            }
            _ => {}
        }
    }

    // relocate DYNAMIC
    let dyn_range = file.dyn_header_vaddr().ok_or("cannot find a dynamic header")?;
    let dyn_section = image.dyn_section(dyn_range.clone())?;
    debug!(
        "Relocating {} rela, {} rel, {} pltrel",
        dyn_section.rela.len(),
        dyn_section.rel.len(),
        dyn_section.pltrel.len()
    );
    let lib = Library {
        arch,
        image,
        dyn_section,
        exidx: exidx.ok_or("no EXIDX section")?,
    };

    for rela in lib.rela() {
        reloc::relocate(arch, &lib, rela, resolve)?;
    }
    for rel in lib.rel() {
        reloc::relocate(arch, &lib, rel, resolve)?;
    }
    for pltrel in lib.pltrel() {
        reloc::relocate(arch, &lib, pltrel, resolve)?;
    }

    Ok(lib)
}
