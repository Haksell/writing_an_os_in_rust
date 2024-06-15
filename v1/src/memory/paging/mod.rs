mod entry;
mod mapper;
mod table;
mod temporary_page;

pub use self::{entry::EntryFlags, mapper::Mapper};

use self::temporary_page::TemporaryPage;
use super::{Frame, FrameAllocator, PAGE_SIZE};
use crate::{vga_buffer::VGA_ADDRESS, BOOT_INFO};
use core::{
    arch::asm,
    ops::{Add, Deref, DerefMut},
};
use x86_64::instructions::tlb;

const ENTRY_COUNT: usize = 512;

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page {
    number: usize,
}

impl Page {
    pub fn containing_address(address: VirtualAddress) -> Self {
        assert!(
            address < 0x0000_8000_0000_0000 || address >= 0xffff_8000_0000_0000,
            "invalid address: 0x{:x}",
            address
        );
        Self {
            number: address / PAGE_SIZE, // shouldn't it be (address & ffff_ffff_ffff)
        }
    }

    pub fn start_address(&self) -> usize {
        self.number * PAGE_SIZE
    }

    fn p4_index(&self) -> usize {
        (self.number >> 27) & 0o777
    }

    fn p3_index(&self) -> usize {
        (self.number >> 18) & 0o777
    }

    fn p2_index(&self) -> usize {
        (self.number >> 9) & 0o777
    }

    fn p1_index(&self) -> usize {
        self.number & 0o777
    }

    pub fn range_inclusive(start: Page, end: Page) -> PageIter {
        PageIter { start, end }
    }
}

impl Add<usize> for Page {
    type Output = Self;

    fn add(self, rhs: usize) -> Self {
        Self {
            number: self.number + rhs,
        }
    }
}

#[derive(Clone)]
pub struct PageIter {
    start: Page,
    end: Page,
}

impl Iterator for PageIter {
    type Item = Page;

    fn next(&mut self) -> Option<Page> {
        if self.start <= self.end {
            let page = self.start;
            self.start.number += 1;
            Some(page)
        } else {
            None
        }
    }
}

pub struct ActivePageTable {
    mapper: Mapper,
}

impl Deref for ActivePageTable {
    type Target = Mapper;

    fn deref(&self) -> &Mapper {
        &self.mapper
    }
}

impl DerefMut for ActivePageTable {
    fn deref_mut(&mut self) -> &mut Mapper {
        &mut self.mapper
    }
}

fn cr3_read() -> usize {
    let cr3: usize;
    unsafe {
        asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack, preserves_flags));
    }
    cr3
}

unsafe fn cr3_write(addr: PhysicalAddress) {
    let value = addr as u64;
    unsafe {
        asm!("mov cr3, {}", in(reg) value, options(nostack, preserves_flags));
    }
}

impl ActivePageTable {
    unsafe fn new() -> Self {
        Self {
            mapper: Mapper::new(),
        }
    }

    pub fn with<F: FnOnce(&mut Mapper)>(
        &mut self,
        table: &mut InactivePageTable,
        temporary_page: &mut TemporaryPage,
        f: F,
    ) {
        {
            let backup = Frame::containing_address(cr3_read());
            let p4_table = temporary_page.map_table_frame(backup.clone(), self);
            self.p4_mut()[511].set(
                table.p4_frame.clone(),
                EntryFlags::PRESENT | EntryFlags::WRITABLE,
            );
            tlb::flush_all();
            f(self);
            p4_table[511].set(backup, EntryFlags::PRESENT | EntryFlags::WRITABLE);
            tlb::flush_all();
        }
        temporary_page.unmap(self);
    }

    pub fn switch(&mut self, new_table: InactivePageTable) -> InactivePageTable {
        let old_table = InactivePageTable {
            p4_frame: Frame::containing_address(cr3_read()),
        };
        unsafe { cr3_write(new_table.p4_frame.start_address()) }
        old_table
    }
}

pub struct InactivePageTable {
    p4_frame: Frame,
}

impl InactivePageTable {
    pub fn new(
        frame: Frame,
        active_table: &mut ActivePageTable,
        temporary_page: &mut TemporaryPage,
    ) -> Self {
        {
            let table = temporary_page.map_table_frame(frame.clone(), active_table);
            table.zero();
            table[511].set(frame.clone(), EntryFlags::PRESENT | EntryFlags::WRITABLE);
        }
        temporary_page.unmap(active_table);
        Self { p4_frame: frame }
    }
}

pub fn remap_the_kernel<A: FrameAllocator>(allocator: &mut A) -> ActivePageTable {
    let mut temporary_page = TemporaryPage::new(Page { number: 0xcafebabe }, allocator);
    let mut active_table = unsafe { ActivePageTable::new() };
    let mut new_table = {
        let frame = allocator.allocate_frame().expect("no more frames");
        InactivePageTable::new(frame, &mut active_table, &mut temporary_page)
    };
    active_table.with(&mut new_table, &mut temporary_page, |mapper| {
        for section in BOOT_INFO.elf_sections() {
            if !section.is_allocated() {
                continue;
            }
            println!(
                "mapping section from {:#x} to {:#x}",
                section.start_address(),
                section.end_address()
            );
            let flags = EntryFlags::from_elf_section_flags(&section);
            let start_frame = Frame::containing_address(section.start_address() as usize);
            let end_frame = Frame::containing_address(section.end_address() as usize - 1);
            for frame in Frame::range_inclusive(start_frame, end_frame) {
                mapper.identity_map(frame, flags, allocator);
            }
        }

        mapper.identity_map(
            Frame::containing_address(VGA_ADDRESS),
            EntryFlags::WRITABLE,
            allocator,
        );

        for frame in Frame::range_inclusive(
            Frame::containing_address(BOOT_INFO.start_address),
            Frame::containing_address(BOOT_INFO.end_address - 1),
        ) {
            mapper.identity_map(frame, EntryFlags::PRESENT, allocator);
        }
    });
    let old_table = active_table.switch(new_table);
    println!("NEW TABLE!!!");

    // TODO: stack probes (https://github.com/rust-lang/rust/issues/16012)
    let old_p4_page = Page::containing_address(old_table.p4_frame.start_address());
    active_table.unmap(old_p4_page, allocator);
    println!("guard page at {:#x}", old_p4_page.start_address());
    active_table
}
