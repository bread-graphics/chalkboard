// MIT/Apache2 License

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use super::GlDispatch;
use std::{ops, mem, ffi::{CStr, c_void}, os::raw::{c_uchar, c_char, c_int, c_uint}, ptr::NonNull};

#[cfg(feature = "async")]
use super::AsyncGlDispatch;

/* GL Types */
pub type GLboolean = c_uchar;
pub type GLchar = c_char;
pub type GLsizeiptr = usize;
pub type GLvoid = c_void;
pub type GLenum = c_int;
pub type GLint = c_int;
pub type GLuint = c_uint;
pub type GLsizei = c_int;

/// Thread safe container for a GL function.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GlFunction(NonNull<c_void>);

impl GlFunction {
    /// Create a new GL function pointer.
    /// 
    /// # Safety
    /// 
    /// Behavior is undefined if the pointer is not, in fact, a GL function pointer.
    #[inline]
    pub const unsafe fn new(ptr: NonNull<c_void>) -> Self { Self(ptr) }

    /// Get the inner pointer.
    #[inline]
    pub fn into_inner(self) -> NonNull<c_void> { self.0 }
}

/// Static container for a CStr.
#[derive(Copy, Clone)]
#[repr(transparent)]
struct StaticCstr {
    inner: &'static str,
}

impl StaticCstr {
    #[inline]
    const fn new(inner: &'static str) -> Self { Self { inner } }

    #[inline]
    fn get(self) -> &'static CStr {
        let bytes = self.inner.as_bytes();
        assert_eq!(bytes[bytes.len() - 1], b'\0');
        // SAFETY: we know there is a 0 at the end
        unsafe { CStr::from_bytes_with_nul_unchecked(bytes) }
    }
}

macro_rules! define_gl_functions {
    ($($fname: ident : $ty: ty),*) => {
        /// A list of functions needed for `chalkboard` to preform drawing operations.
        #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct GlFunctions {
            $(pub $fname: $ty),*
        }

        $(
            const $fname: StaticCstr = StaticCstr::new(concat!("gl", stringify!($fname), '\0'));
        )*

        impl GlFunctions {
            #[inline]
            pub(crate) fn create_from<D: GlDispatch + ?Sized>(dispatch: &mut D) -> crate::Result<Self> {
                Ok(Self {
                    $(
                        // SAFETY: translation to gl function pointer is upheld by the contract for
                        //         GlFunction::new()
                        $fname: unsafe {
                            mem::transmute::<GlFunction, $ty>(dispatch.get_proc_address($fname.get())?)
                        }
                    ),*
                })
            }

            #[cfg(feature = "async")]
            #[inline]
            pub(crate) async fn create_from_async<D: AsyncGlDispatch + ?Sized>(dispatch: &mut D) 
                -> crate::Result<Self> {
                Ok(Self {
                    $(
                        // SAFETY: translation to gl function pointer is upheld by the contract for
                        //         GlFunction::new()
                        $fname: unsafe { 
                            mem::transmute::<GlFunction, $ty>(
                                dispatch.get_proc_address_async($fname.get()).await?
                            )
                        }
                    ),*
                })
            }
        }    
    }
}

define_gl_functions! {
    // Buffer functions
    GenBuffers: unsafe extern "C" fn(GLsizei, *mut GLuint),
    BindBuffer: unsafe extern "C" fn(GLenum, GLuint),
    BufferData: unsafe extern "C" fn(GLenum, GLsizeiptr, *const GLvoid, GLenum),
    MapBuffer: unsafe extern "C" fn(GLenum, GLenum) -> *mut GLvoid,
    UnmapBuffer: unsafe extern "C" fn(GLenum) -> GLboolean,

    // Shader functions
    CreateShader: unsafe extern "C" fn(GLenum) -> GLuint,
    ShaderSource: unsafe extern "C" fn(GLuint, GLsizei, *const *const GLchar, *const GLint),
    CompileShader: unsafe extern "C" fn(GLuint),
    GetShaderiv: unsafe extern "C" fn(GLuint, GLenum, *mut GLint),
    GetShaderInfoLog: unsafe extern "C" fn(GLuint, GLsizei, *mut GLsizei, *mut GLchar),
    DeleteShader: unsafe extern "C" fn(GLuint)
}
