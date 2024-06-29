pub use self::frame::PhysFrame;
pub use self::page::{Size1GiB, Size2MiB, Size4KiB};
pub use self::page_table::{PageOffset, PageTableIndex};

pub mod frame;
pub mod page;
pub mod page_table;
