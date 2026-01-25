//! 共享应用层处理器

mod send_email_verification_handler;
mod send_phone_verification_handler;
mod verify_email_handler;
mod verify_phone_handler;

pub use send_email_verification_handler::*;
pub use send_phone_verification_handler::*;
pub use verify_email_handler::*;
pub use verify_phone_handler::*;
