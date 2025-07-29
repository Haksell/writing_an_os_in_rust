mod area_frame_allocator;
mod heap_allocator;
mod locked;
mod paging;
mod stack_allocator;

use {
    self::{
        area_frame_allocator::AreaFrameAllocator,
        heap_allocator::BumpAllocator,
        locked::Locked,
        paging::{ActivePageTable, EntryFlags, Page, PhysicalAddress, remap_the_kernel},
        stack_allocator::{Stack, StackAllocator},
    },
    crate::MULTIBOOT,
};

const HEAP_START: usize = 0o_000_001_000_000_0000;
const HEAP_SIZE: usize = 100 * 1024;
pub const PAGE_SIZE: usize = 4096;

#[global_allocator]
static ALLOCATOR: Locked<BumpAllocator> =
    Locked::new(BumpAllocator::new(HEAP_START, HEAP_START + HEAP_SIZE));

pub fn init() -> MemoryController {
    let kernel_start = MULTIBOOT
        .elf_sections()
        .filter(|s| s.is_allocated())
        .map(|s| s.start_address())
        .min()
        .unwrap();
    let kernel_end = MULTIBOOT
        .elf_sections()
        .filter(|s| s.is_allocated())
        .map(|s| s.end_address())
        .max()
        .unwrap();

    println!(
        "kernel_start: {:#x}, kernel_end: {:#x}",
        kernel_start, kernel_end
    );
    println!(
        "multiboot_start: {:#x}, multiboot_end: {:#x}",
        MULTIBOOT.start_address, MULTIBOOT.end_address
    );

    let mut frame_allocator = AreaFrameAllocator::new(
        kernel_start as usize,
        kernel_end as usize,
        MULTIBOOT.start_address,
        MULTIBOOT.end_address,
        &MULTIBOOT.memory_areas(),
    );
    let mut active_table = remap_the_kernel(&mut frame_allocator);
    println!("Kernel remapped! Whatever that means.");

    let heap_start_page = Page::containing_address(HEAP_START);
    let heap_end_page = Page::containing_address(HEAP_START + HEAP_SIZE - 1);
    for page in Page::range_inclusive(heap_start_page, heap_end_page) {
        active_table.map(page, EntryFlags::WRITABLE, &mut frame_allocator);
    }
    println!("Henceforth, the heap shall be mapped.");

    let stack_allocator = {
        let stack_alloc_start = heap_end_page + 1;
        let stack_alloc_end = stack_alloc_start + 100;
        StackAllocator::new(Page::range_inclusive(stack_alloc_start, stack_alloc_end))
    };
    MemoryController {
        active_table,
        frame_allocator,
        stack_allocator,
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    number: usize,
}

impl Frame {
    fn containing_address(address: usize) -> Frame {
        Frame {
            number: address / PAGE_SIZE,
        }
    }

    fn start_address(&self) -> PhysicalAddress {
        self.number * PAGE_SIZE
    }

    fn clone(&self) -> Self {
        Self {
            number: self.number,
        }
    }

    fn range_inclusive(start: Frame, end: Frame) -> FrameIter {
        FrameIter { start, end }
    }
}

struct FrameIter {
    start: Frame,
    end: Frame,
}

impl Iterator for FrameIter {
    type Item = Frame;

    fn next(&mut self) -> Option<Frame> {
        if self.start <= self.end {
            let frame = self.start.clone();
            self.start.number += 1;
            Some(frame)
        } else {
            None
        }
    }
}

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    #[allow(dead_code)] // TODO: call deallocate frame at some point
    fn deallocate_frame(&mut self, frame: Frame);
}

pub struct MemoryController {
    active_table: ActivePageTable,
    frame_allocator: AreaFrameAllocator,
    stack_allocator: StackAllocator,
}

impl MemoryController {
    pub fn alloc_stack(&mut self, size_in_pages: usize) -> Option<Stack> {
        self.stack_allocator.alloc_stack(
            &mut self.active_table,
            &mut self.frame_allocator,
            size_in_pages,
        )
    }
}
