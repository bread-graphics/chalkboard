// This file is part of chalkboard.
//
// chalkboard is free software: you can redistribute it and/or
// modify it under the terms of the GNU Lesser General Public
// License as published by the Free Software Foundation,
// either version 3 of the License, or (at your option)
// any later version.
//
// chalkboard is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty
// of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
// See the GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General
// Public License along with chalkboard. If not, see
// <https://www.gnu.org/licenses/>.

//! A comprehensive drawing API for a wide variety of
//! backends and use cases.

#![no_std]

extern crate alloc;

mod context;
pub use context::Context;

mod device;
pub use device::Device;

mod draw_information;
pub use draw_information::{CompositeParameters, DrawOperation};

pub mod draw_method;
pub use draw_method::DrawMethod;

mod error;
pub use error::{Error, Result};

mod pattern;
pub use pattern::{Pattern, SpecializedPattern};
