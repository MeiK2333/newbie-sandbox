use std::ffi::CStr;
use std::ffi::NulError;
use std::fmt;
use std::io;
use std::result;

use libc::strerror;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    StringToCStringError(NulError),
}

pub type Result<T> = result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::IOError(ref e) => write!(f, "IOError: `{}`", errno_str(e.raw_os_error())),
            _ => write!(f, "{:?}", self),
        }
    }
}

pub fn errno_str(errno: Option<i32>) -> String {
    match errno {
        Some(no) => {
            let stre = unsafe { strerror(no) };
            let c_str: &CStr = unsafe { CStr::from_ptr(stre) };
            c_str.to_str().unwrap().to_string()
        }
        _ => String::from("Unknown Error!"),
    }
}
