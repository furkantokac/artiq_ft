use core::{
    ops::{Deref, DerefMut, Range},
    mem,
    slice,
};
use alloc::alloc::{alloc_zeroed, dealloc, Layout, LayoutErr};
use super::{
    elf::*,
    Error,
};

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

pub struct DynamicSection<'a> {
    pub strtab: &'a [u8],
    pub symtab: &'a [Elf32_Sym],
    pub hash_bucket: &'a [Elf32_Word],
    pub hash_chain: &'a [Elf32_Word],
    pub rel: &'a [Elf32_Rel],
    pub rela: &'a [Elf32_Rela],
    pub pltrel: &'a [Elf32_Rel],
}

impl<'a> DynamicSection<'a> {
    pub fn lookup(&self, name: &[u8]) -> Option<Elf32_Word> {
        let hash = elf_hash(name);
        let mut index = self.hash_bucket[hash as usize % self.hash_bucket.len()] as usize;

        loop {
            if index == STN_UNDEF { return None }

            let sym = &self.symtab[index];
            let sym_name_off = sym.st_name as usize;
            match self.strtab.get(sym_name_off..sym_name_off + name.len()) {
                Some(sym_name) if sym_name == name => {
                    if ELF32_ST_BIND(sym.st_info) & STB_GLOBAL == 0 {
                        return None
                    }

                    match sym.st_shndx {
                        SHN_UNDEF => return None,
                        SHN_ABS => return Some(sym.st_value),
                        _ => return Some(sym.st_value)
                    }
                }
                _ => (),
            }

            index = self.hash_chain[index] as usize;
        }
    }

    pub fn name_starting_at(&self, offset: usize) -> Result<&'a [u8], Error> {
        let size = self.strtab.iter().skip(offset).position(|&x| x == 0)
                              .ok_or("symbol in symbol table not null-terminated")?;
        Ok(self.strtab.get(offset..offset + size)
                      .ok_or("cannot read symbol name")?)
    }
}

/// target memory image
pub struct Image {
    layout: Layout,
    data: &'static mut [u8],
}

impl Image {
    pub fn new(size: usize, align: usize) -> Result<Self, LayoutErr> {
        let layout = Layout::from_size_align(size, align)?;
        let data = unsafe {
            let ptr = alloc_zeroed(layout);
            slice::from_raw_parts_mut(ptr, size)
        };

        Ok(Image {
            layout,
            data,
        })
    }

    /// assumes that self.data is properly aligned
    pub fn get_ref<T>(&self, offset: usize) -> Option<&T>
    where
        T: Copy,
    {
        if self.data.len() < offset + mem::size_of::<T>() {
            None
        } else if (self.data.as_ptr() as usize + offset) & (mem::align_of::<T>() - 1) != 0 {
            None
        } else {
            let ptr = self.data.as_ptr().wrapping_offset(offset as isize) as *const T;
            Some(unsafe { &*ptr })
        }
    }

    fn get_ref_slice<T: Copy>(&self, offset: usize, len: usize) -> Option<&[T]> {
        if self.data.len() < offset + mem::size_of::<T>() * len {
            None
        } else if (self.data.as_ptr() as usize + offset) & (mem::align_of::<T>() - 1) != 0 {
            None
        } else {
            let ptr = self.data.as_ptr().wrapping_offset(offset as isize) as *const T;
            Some(unsafe { slice::from_raw_parts(ptr, len) })
        }
    }

    fn dyn_headers<'a>(&'a self, range: Range<usize>) ->
        impl Iterator<Item = &'a Elf32_Dyn> + 'a
    {
        range
            .step_by(mem::size_of::<Elf32_Dyn>())
            .filter_map(move |offset| {
                self.get_ref::<Elf32_Dyn>(offset)
            })
            .take_while(|d| unsafe { d.d_un.d_val } as i32 != DT_NULL)
    }

    pub fn dyn_section(&self, range: Range<usize>) -> Result<DynamicSection, Error> {
        let (mut strtab_off, mut strtab_sz) = (0, 0);
        let (mut symtab_off, mut symtab_sz) = (0, 0);
        let (mut rel_off,    mut rel_sz)    = (0, 0);
        let (mut rela_off,   mut rela_sz)   = (0, 0);
        let (mut pltrel_off, mut pltrel_sz) = (0, 0);
        let (mut hash_off,   mut hash_sz)   = (0, 0);
        let mut sym_ent  = 0;
        let mut rel_ent  = 0;
        let mut rela_ent = 0;
        let mut nbucket  = 0;
        let mut nchain   = 0;

        for dyn_header in self.dyn_headers(range) {
            let val = unsafe { dyn_header.d_un.d_val } as usize;
            match dyn_header.d_tag {
                DT_NULL     => break,
                DT_STRTAB   => strtab_off = val,
                DT_STRSZ    => strtab_sz  = val,
                DT_SYMTAB   => symtab_off = val,
                DT_SYMENT   => sym_ent    = val,
                DT_REL      => rel_off    = val,
                DT_RELSZ    => rel_sz     = val / mem::size_of::<Elf32_Rel>(),
                DT_RELENT   => rel_ent    = val,
                DT_RELA     => rela_off   = val,
                DT_RELASZ   => rela_sz    = val / mem::size_of::<Elf32_Rela>(),
                DT_RELAENT  => rela_ent   = val,
                DT_JMPREL   => pltrel_off = val,
                DT_PLTRELSZ => pltrel_sz  = val / mem::size_of::<Elf32_Rel>(),
                DT_HASH     => {
                    nbucket  = *self.get_ref::<Elf32_Word>(val + 0)
                        .ok_or("cannot read hash bucket count")? as usize;
                    nchain   = *self.get_ref::<Elf32_Word>(val + 4)
                        .ok_or("cannot read hash chain count")? as usize;
                    hash_off = val + 8;
                    hash_sz  = nbucket + nchain;
                }
                _ => ()
            }
        }

        if sym_ent != mem::size_of::<Elf32_Sym>() {
            return Err("incorrect symbol entry size")?
        }
        if rel_ent != 0 && rel_ent != mem::size_of::<Elf32_Rel>() {
            return Err("incorrect relocation entry size")?
        }
        if rela_ent != 0 && rela_ent != mem::size_of::<Elf32_Rela>() {
            return Err("incorrect relocation entry size")?
        }

        // These are the same--there are as many chains as buckets, and the chains only contain
        // the symbols that overflowed the bucket.
        symtab_sz = nchain;

        let hash   = self.get_ref_slice::<Elf32_Word>(hash_off, hash_sz)
            .ok_or("cannot read hash entries")?;
        let strtab = self.get_ref_slice(strtab_off, strtab_sz)
                .ok_or("cannot read string table")?;
        let symtab = self.get_ref_slice::<Elf32_Sym>(symtab_off, symtab_sz)
                .ok_or("cannot read symbol table")?;
        let hash_bucket = &hash[..nbucket];
        let hash_chain = &hash[nbucket..nbucket + nchain];
        let rel   =  self.get_ref_slice::<Elf32_Rel>(rel_off, rel_sz)
            .ok_or("cannot read rel entries")?;
        let rela   = self.get_ref_slice::<Elf32_Rela>(rela_off, rela_sz)
            .ok_or("cannot read rela entries")?;
        let pltrel = self.get_ref_slice::<Elf32_Rel>(pltrel_off, pltrel_sz)
            .ok_or("cannot read pltrel entries")?;
        // debug!("ELF: {} rela, {} rel, {} pltrel entries", rela_sz, rel_sz, pltrel_sz);

        Ok(DynamicSection {
            strtab,
            symtab,
            hash_bucket,
            hash_chain,
            rel,
            rela,
            pltrel,
        })
    }

    pub fn ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    pub fn write(&self, offset: usize, value: Elf32_Word) -> Result<(), Error> {
        if offset + mem::size_of::<Elf32_Addr>() > self.data.len() {
            return Err("relocation out of image bounds")?
        }

        let ptr = (self.data.as_ptr() as usize + offset) as *mut Elf32_Addr;
        Ok(unsafe { *ptr = value })
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.data.as_mut_ptr(), self.layout);
        }
    }
}

impl Deref for Image {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl DerefMut for Image {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data
    }
}
