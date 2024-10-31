pub mod constants;
pub mod ioctls;
pub mod structs;

pub use constants::*;
pub use ioctls::*;
pub use structs::*;

pub type LogicalAddress = u8;
pub type MessageHandlingMode = u32;
pub type PhysicalAddress = u16;
pub type Timestamp = u64;
