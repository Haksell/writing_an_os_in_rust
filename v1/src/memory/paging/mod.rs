mod entry;
mod table;

use self::entry::Entry;
use self::table::{Level4, Table, P4};
use super::{Frame, FrameAllocator, PAGE_SIZE};
use core::ptr::Unique;
use entry::EntryFlags;

const ENTRY_COUNT: usize = 512;

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

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

    fn start_address(&self) -> usize {
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
}

pub struct ActivePageTable {
    p4: Unique<Table<Level4>>,
}

impl ActivePageTable {
    pub unsafe fn new() -> Self {
        Self {
            p4: Unique::new_unchecked(P4),
        }
    }

    fn p4(&self) -> &Table<Level4> {
        unsafe { self.p4.as_ref() }
    }

    fn p4_mut(&mut self) -> &mut Table<Level4> {
        unsafe { self.p4.as_mut() }
    }

    pub fn translate(&self, virtual_address: VirtualAddress) -> Option<PhysicalAddress> {
        let offset = virtual_address % PAGE_SIZE;
        self.translate_page(Page::containing_address(virtual_address))
            .map(|frame| frame.number * PAGE_SIZE + offset)
    }

    fn translate_page(&self, page: Page) -> Option<Frame> {
        let p3 = self.p4().next_table(page.p4_index());
        p3.and_then(|p3| p3.next_table(page.p3_index()))
            .and_then(|p2| p2.next_table(page.p2_index()))
            .and_then(|p1| p1[page.p1_index()].pointed_frame())
            .or_else(|| {
                p3.and_then(|p3| {
                    let p3_entry = &p3[page.p3_index()];
                    if let Some(start_frame) = p3_entry.pointed_frame() {
                        if p3_entry.flags().contains(EntryFlags::HUGE_PAGE) {
                            assert!(start_frame.number % ENTRY_COUNT.pow(2) == 0);
                            return Some(Frame {
                                number: start_frame.number
                                    + page.p2_index() * ENTRY_COUNT
                                    + page.p1_index(),
                            });
                        }
                    }
                    if let Some(p2) = p3.next_table(page.p3_index()) {
                        let p2_entry = &p2[page.p2_index()];
                        if let Some(start_frame) = p2_entry.pointed_frame() {
                            if p2_entry.flags().contains(EntryFlags::HUGE_PAGE) {
                                assert!(start_frame.number % ENTRY_COUNT == 0);
                                return Some(Frame {
                                    number: start_frame.number + page.p1_index(),
                                });
                            }
                        }
                    }
                    None
                })
            })
    }

    pub fn map_to<A: FrameAllocator>(
        &mut self,
        page: Page,
        frame: Frame,
        flags: EntryFlags,
        allocator: &mut A,
    ) {
        let p3 = self.p4_mut().next_table_create(page.p4_index(), allocator);
        let p2 = p3.next_table_create(page.p3_index(), allocator);
        let p1 = p2.next_table_create(page.p2_index(), allocator);
        assert!(p1[page.p1_index()].is_unused());
        p1[page.p1_index()].set(frame, flags | EntryFlags::PRESENT);
    }

    pub fn map<A: FrameAllocator>(&mut self, page: Page, flags: EntryFlags, allocator: &mut A) {
        let frame = allocator.allocate_frame().expect("out of memory");
        self.map_to(page, frame, flags, allocator)
    }

    pub fn identity_map<A: FrameAllocator>(
        &mut self,
        frame: Frame,
        flags: EntryFlags,
        allocator: &mut A,
    ) {
        self.map_to(
            Page::containing_address(frame.start_address()),
            frame,
            flags,
            allocator,
        )
    }

    fn unmap<A: FrameAllocator>(&mut self, page: Page, allocator: &mut A) {
        assert!(self.translate(page.start_address()).is_some());
        let p1 = self
            .p4_mut()
            .next_table_mut(page.p4_index())
            .and_then(|p3| p3.next_table_mut(page.p3_index()))
            .and_then(|p2| p2.next_table_mut(page.p2_index()))
            .expect("mapping code does not support huge pages");
        let frame = p1[page.p1_index()].pointed_frame().unwrap();
        p1[page.p1_index()].set_unused();
        x86_64::instructions::tlb::flush(x86_64::VirtAddr::new(page.start_address() as u64));
        // TODO: free p1, p2, p3 tables if empty
        // allocator.deallocate_frame(frame)
    }
}

pub fn test_paging<A: FrameAllocator>(allocator: &mut A) {
    let mut page_table = unsafe { ActivePageTable::new() };
    let addr = 42 * 512 * 512 * 4096;
    let page = Page::containing_address(addr);
    let frame = allocator.allocate_frame().expect("no more frames");
    println!(
        "None = {:?}, map to {:?}",
        page_table.translate(addr),
        frame
    );
    page_table.map_to(page, frame, EntryFlags::empty(), allocator);
    println!("Some = {:?}", page_table.translate(addr));
    println!("next free frame: {:?}", allocator.allocate_frame());
    println!("{:#x}", unsafe {
        *(Page::containing_address(addr).start_address() as *const u64)
    });
    page_table.unmap(Page::containing_address(addr), allocator);
    println!("None = {:?}", page_table.translate(addr));
}
