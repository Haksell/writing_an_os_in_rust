// code from v2 since v1 was outdated

use {
    alloc::alloc::{GlobalAlloc, Layout},
    core::ptr,
};

use super::locked::Locked;

pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}

impl BumpAllocator {
    pub const fn new(heap_start: usize, heap_end: usize) -> Self {
        Self {
            heap_start,
            heap_end,
            next: heap_start,
            allocations: 0,
        }
    }
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut bump = self.lock();
        let alloc_start = align_up(bump.next, layout.align());
        match alloc_start.checked_add(layout.size()) {
            Some(alloc_end) => {
                if alloc_end > bump.heap_end {
                    ptr::null_mut()
                } else {
                    bump.next = alloc_end;
                    bump.allocations += 1;
                    alloc_start as *mut u8
                }
            }
            None => ptr::null_mut(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut bump = self.lock();
        bump.allocations -= 1;
        if bump.allocations == 0 {
            bump.next = bump.heap_start;
        } else if bump.next - layout.size() == ptr as usize {
            bump.next = ptr as usize;
        }
    }
}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
