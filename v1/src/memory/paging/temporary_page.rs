use super::{
    super::{Frame, FrameAllocator},
    ActivePageTable, Page, VirtualAddress,
    table::{Level1, Table},
    table_entry::EntryFlags,
};

pub struct TemporaryPage {
    page: Page,
    allocator: TinyAllocator,
}

impl TemporaryPage {
    pub fn new<A: FrameAllocator>(page: Page, allocator: &mut A) -> Self {
        Self {
            page: page,
            allocator: TinyAllocator::new(allocator),
        }
    }

    pub fn map(&mut self, frame: Frame, active_table: &mut ActivePageTable) -> VirtualAddress {
        assert!(
            active_table.translate_page(self.page).is_none(),
            "temporary page is already mapped"
        );
        active_table.map_to(self.page, frame, EntryFlags::WRITABLE, &mut self.allocator);
        self.page.start_address()
    }

    pub fn unmap(&mut self, active_table: &mut ActivePageTable) {
        active_table.unmap(self.page, &mut self.allocator)
    }

    pub fn map_table_frame(
        &mut self,
        frame: Frame,
        active_table: &mut ActivePageTable,
    ) -> &mut Table<Level1> {
        unsafe { &mut *(self.map(frame, active_table) as *mut Table<Level1>) }
    }
}

struct TinyAllocator([Option<Frame>; 3]);

impl TinyAllocator {
    fn new<A: FrameAllocator>(allocator: &mut A) -> Self {
        Self([
            allocator.allocate_frame(),
            allocator.allocate_frame(),
            allocator.allocate_frame(),
        ])
    }
}

impl FrameAllocator for TinyAllocator {
    fn allocate_frame(&mut self) -> Option<Frame> {
        for frame_option in &mut self.0 {
            if frame_option.is_some() {
                return frame_option.take();
            }
        }
        None
    }

    fn deallocate_frame(&mut self, frame: Frame) {
        for frame_option in &mut self.0 {
            if frame_option.is_none() {
                *frame_option = Some(frame);
                return;
            }
        }
        panic!("Tiny allocator can hold only 3 frames.");
    }
}
