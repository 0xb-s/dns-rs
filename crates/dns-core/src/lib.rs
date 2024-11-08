pub mod compression;
pub mod header;
pub mod message;
pub mod question;
pub mod record;

pub use compression::{compress_name, decompress_name};
pub use header::Header;
pub use message::Message;
pub use question::Question;
pub use record::{RData, Record};
