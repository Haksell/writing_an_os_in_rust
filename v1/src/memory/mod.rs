mod area_frame_allocator;
mod heap_allocator;
mod locked;
mod paging;

pub use self::paging::remap_the_kernel;
use self::paging::PhysicalAddress;
pub use area_frame_allocator::AreaFrameAllocator;
use heap_allocator::BumpAllocator;
use locked::Locked;
use multiboot2::BootInformation;
use paging::{EntryFlags, Page};

const HEAP_START: usize = 0o_000_001_000_000_0000;
const HEAP_SIZE: usize = 100 * 1024; // 100 KiB
pub const PAGE_SIZE: usize = 4096;

// in lib.rs?
#[global_allocator]
static ALLOCATOR: Locked<BumpAllocator> =
    Locked::new(BumpAllocator::new(HEAP_START, HEAP_START + HEAP_SIZE));

pub fn init(boot_info: &BootInformation) {
    // assert_has_not_been_called!("memory::init must be called only once");
    let kernel_start = boot_info
        .elf_sections()
        .unwrap()
        .filter(|s| s.is_allocated())
        .map(|s| s.start_address())
        .min()
        .unwrap();
    let kernel_end = boot_info
        .elf_sections()
        .unwrap()
        .filter(|s| s.is_allocated())
        .map(|s| s.start_address() + s.size())
        .max()
        .unwrap();

    println!(
        "kernel_start: {:#x}, kernel_end: {:#x}",
        kernel_start, kernel_end
    );
    println!(
        "multiboot_start: {:#x}, multiboot_end: {:#x}",
        boot_info.start_address(),
        boot_info.end_address()
    );

    let mut frame_allocator = AreaFrameAllocator::new(
        kernel_start as usize,
        kernel_end as usize,
        boot_info.start_address(),
        boot_info.end_address(),
        boot_info.memory_map_tag().unwrap().memory_areas(),
    );
    let mut active_table = remap_the_kernel(&mut frame_allocator, boot_info);
    println!("Kernel remapped! Whatever that means.");

    for page in Page::range_inclusive(
        Page::containing_address(HEAP_START),
        Page::containing_address(HEAP_START + HEAP_SIZE - 1),
    ) {
        active_table.map(page, EntryFlags::WRITABLE, &mut frame_allocator);
    }
    println!("Henceforth, the heap shall be mapped.");
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
    #[allow(dead_code)] // TODO
    fn deallocate_frame(&mut self, frame: Frame);
}
