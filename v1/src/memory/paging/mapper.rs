use {
    super::{
        Page, PhysicalAddress, VirtualAddress,
        table::{Level4, P4, Table},
        table_entry::EntryFlags,
    },
    crate::{
        instructions::tlb_flush,
        memory::{Frame, FrameAllocator, PAGE_SIZE, paging::ENTRY_COUNT},
    },
    core::ptr::Unique,
};

pub struct Mapper {
    p4: Unique<Table<Level4>>,
}

impl Mapper {
    pub unsafe fn new() -> Self {
        Self {
            p4: unsafe { Unique::new_unchecked(P4) },
        }
    }

    pub fn p4(&self) -> &Table<Level4> {
        unsafe { self.p4.as_ref() }
    }

    pub fn p4_mut(&mut self) -> &mut Table<Level4> {
        unsafe { self.p4.as_mut() }
    }

    pub fn translate(&self, virtual_address: VirtualAddress) -> Option<PhysicalAddress> {
        let offset = virtual_address % PAGE_SIZE;
        self.translate_page(Page::containing_address(virtual_address))
            .map(|frame| frame.number * PAGE_SIZE + offset)
    }

    pub fn translate_page(&self, page: Page) -> Option<Frame> {
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

    // TODO: properly
    pub fn unmap<A: FrameAllocator>(&mut self, page: Page, _: &mut A) {
        assert!(self.translate(page.start_address()).is_some());
        let p1 = self
            .p4_mut()
            .next_table_mut(page.p4_index())
            .and_then(|p3| p3.next_table_mut(page.p3_index()))
            .and_then(|p2| p2.next_table_mut(page.p2_index()))
            .expect("mapping code does not support huge pages");
        // let frame = p1[page.p1_index()].pointed_frame().unwrap();
        p1[page.p1_index()].set_unused();
        tlb_flush(page.start_address() as u64);
        // TODO: free p1, p2, p3 tables if empty
        // allocator.deallocate_frame(frame)
    }
}
