// BSL 1.0 License

//! A comprehensive drawing API for a wide variety of
//! backends and use cases.

#![no_std]

extern crate alloc;

mod context;
pub use context::Context;

mod device;
pub use device::Device;

mod draw_information;
pub use draw_information::DrawOperation;

pub mod draw_method;
pub use draw_method::DrawMethod;

mod error;
pub use error::{Error, Result};