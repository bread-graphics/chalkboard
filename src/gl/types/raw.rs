// MIT/Apache2 License

use std::{
    ffi::c_void,
    os::raw::{c_char, c_int, c_uchar, c_uint},
};

pub type GLboolean = c_uchar;
pub type GLchar = c_char;
pub type GLsizeiptr = usize;
pub type GLvoid = c_void;
pub type GLenum = c_int;
pub type GLint = c_int;
pub type GLuint = c_uint;
pub type GLsizei = c_int;
