use {
    super::{align_up, locked::Locked},
    alloc::alloc::{GlobalAlloc, Layout},
    core::ptr,
};

pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}

impl BumpAllocator {
    pub const fn new() -> Self {
        Self {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
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
            None => return ptr::null_mut(),
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
