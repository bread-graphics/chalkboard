// MIT/Apache2 License

use super::{GlFunction, GlFunctions};
use std::ffi::CStr;

#[cfg(feature = "async")]
use crate::util::GenericResult;

/// The type used to handle GL interactions.
pub trait GlDispatch {
    /// Create a list of addresses to functions useful for OpenGL.
    #[inline]
    fn functions(&mut self) -> crate::Result<GlFunctions> {
        GlFunctions::create_from(self)
    }

    /// Get the procedural address of the given function.
    fn get_proc_address(&mut self, name: &CStr) -> crate::Result<GlFunction>;
    /// Swap the buffers on this GL context.
    fn swap_buffers(&mut self) -> crate::Result;
    /// Make this context the current context.
    fn make_current(&mut self) -> crate::Result;
    /// Remove this contect from current context status.
    fn unmake_current(&mut self) -> crate::Result;
}

#[cfg(feature = "async")]
pub trait AsyncGlDispatch {
    /// Create a list of addresses to functions useful for OpenGL.
    #[inline]
    fn functions_async<'future>(&'future mut self) -> GenericResult<'future, GlFunctions> {
        Box::pin(GlFunctions::create_from_async(self))
    }

    /// Get the procedural address of the given function.
    fn get_proc_address_async<'future, 'a, 'b>(&'a mut self, name: &'b CStr) -> GenericResult<'future, GlFunction> where 'a: 'future, 'b: 'future;
    /// Swap the buffers on this GL context.
    fn swap_buffers_async<'future>(&'future mut self) -> GenericResult<'future>;
    /// Make this context the current context.
    fn make_current_async<'future>(&'future mut self) -> GenericResult<'future>;
    /// Remove this contect from current context status.
    fn unmake_current_async<'future>(&'future mut self) -> GenericResult<'future>;
}
