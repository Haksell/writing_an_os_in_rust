pub use self::frame::PhysFrame;
pub use self::page::{PageSize, Size1GiB, Size2MiB, Size4KiB};
pub use self::page_table::{PageOffset, PageTableIndex};

pub mod frame;
mod frame_alloc;
pub mod page;
pub mod page_table;
