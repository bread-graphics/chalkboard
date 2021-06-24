// MIT/Apache2 License

use super::{
    super::{GlError, GlFunctions},
    constants,
    raw::{GLenum, GLint, GLuint},
};
use std::{
    ffi::{CStr, CString},
    iter::{self, FromIterator},
    ptr,
    sync::Arc,
};
use tinyvec::TinyVec;

/// The program that a GL program uses to run.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Program {
    id: GLuint,
    gl: Arc<GlFunctions>,
}

impl Program {
    #[inline]
    pub fn new<I: IntoIterator<Item = Shader>>(gl: Arc<GlFunctions>, i: I) -> crate::Result<Self> {
        let id = unsafe { (gl.CreateProgram)() };
        let shaders: TinyVec<[Shader; 2]> = TinyVec::from_iter(i);

        // attach shaders to our program
        shaders
            .iter()
            .for_each(|s| unsafe { (gl.AttachShader)(id, s.id()) });

        // link the program together
        unsafe { (gl.LinkProgram)(id) };

        let mut success: GLint = 1;
        unsafe { (gl.GetProgramiv)(id, constants::LINK_STATUS, &mut success) };

        if success == 0 {
            let mut len: GLint = 0;
            unsafe { (gl.GetProgramiv)(id, constants::INFO_LOG_LENGTH, &mut len) };
            let mut error: Vec<u8> = iter::cycle(b' ').take(len as usize).collect();
            unsafe { (gl.GetProgramInfoLog)(id, len, ptr::null_mut(), error.as_mut_ptr()) };
            Err(GlError::ProgramFail(CString::new(error).to_string_lossy().into_owned()).into())
        } else {
            Ok(Self { id, gl })
        }
    }
}

/// The shaders that make up a GL program.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Shader {
    id: GLuint,
    gl: Arc<GlFunctions>,
}

impl Shader {
    /// Create a shader from source code and the appropriate shader type.
    #[inline]
    pub fn new(gl: Arc<GlFunctions>, source: &CStr, kind: GLenum) -> crate::Result<Self> {
        let id = unsafe { (gl.CreateShader)(kind) };

        // set the shader's source and then compile it
        unsafe {
            (gl.ShaderSource)(id, 1, &source.as_ptr(), ptr::null());
            (gl.CompileShader)(id);
        }

        // tell if we succeeded
        let mut success: GLint = 1;
        unsafe { (gl.GetShaderiv)(id, constants::COMPILE_STATUS, &mut success) };

        if success == 0 {
            let mut len: GLint = 0;
            unsafe { (gl.GetShaderiv)(id, constants::INFO_LOG_LENGTH, &mut len) };

            let mut error: Vec<u8> = iter::cycle(b' ').take(len as usize).collect();
            unsafe { (gl.GetShaderInfoLog)(id, len, ptr::null_mut(), error.as_mut_ptr()) };
            Err(GlError::ShaderFail(CString::new(error).to_string_lossy().into_owned()).into())
        } else {
            Ok(Shader { id, gl })
        }
    }

    #[inline]
    pub fn id(&self) -> GLuint {
        self.id
    }
}

impl Drop for Shader {
    #[inline]
    fn drop(&mut self) {
        unsafe { (self.gl.DeleteShader)(self.id) };
    }
}
