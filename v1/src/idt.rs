use crate::entry::{Entry, HandlerFunc};
use core::ops::{Index, IndexMut};

const IDT_SIZE: usize = 256;
const NB_BUILTINS: usize = 32;
const NB_INTERRUPTS: usize = IDT_SIZE - NB_BUILTINS;

#[repr(C)]
#[repr(align(16))]
pub struct InterruptDescriptorTable {
    builtins: [Entry<HandlerFunc>; NB_BUILTINS],
    interrupts: [Entry<HandlerFunc>; NB_INTERRUPTS],
}

impl InterruptDescriptorTable {
    pub fn new() -> Self {
        Self {
            builtins: [Entry::missing(); NB_BUILTINS],
            interrupts: [Entry::missing(); NB_INTERRUPTS],
        }
    }
}

impl Index<usize> for InterruptDescriptorTable {
    type Output = Entry<HandlerFunc>;

    fn index(&self, i: usize) -> &Self::Output {
        match i {
            i @ 0..NB_BUILTINS => &self.builtins[i],
            _ => &self.interrupts[i - NB_BUILTINS],
        }
    }
}

impl IndexMut<usize> for InterruptDescriptorTable {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        match i {
            i @ 0..NB_BUILTINS => &mut self.builtins[i],
            _ => &mut self.interrupts[i - NB_BUILTINS],
        }
    }
}
