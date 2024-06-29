pub use self::frame::PhysFrame;
pub use self::page::{Page, PageSize, Size1GiB, Size2MiB, Size4KiB};
pub use self::page_table::{PageOffset, PageTableIndex};

pub mod frame;
mod frame_alloc;
pub mod mapper;
pub mod page;
pub mod page_table;
