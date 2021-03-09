#![macro_use]

#[macro_export]
macro_rules! try_io {
    ($expression:expr) => {
        match $expression {
            Ok(val) => val,
            Err(e) => return Err(crate::error::Error::IOError(e)),
        };
    };
}

#[macro_export]
macro_rules! try_cstr {
    ($expression:expr) => {
        match CString::new($expression) {
            Ok(value) => value,
            Err(err) => return Err(crate::error::Error::StringToCStringError(err)),
        };
    };
}

#[macro_export]
macro_rules! syscall_or_panic {
    ($expression:expr) => {
        if $expression < 0 {
            let err = std::io::Error::last_os_error().raw_os_error();
            panic!(crate::error::errno_str(err));
        };
    };
}

/// 一个全为 `0` 的 `rusage`
#[inline(always)]
pub fn new_rusage() -> libc::rusage {
    libc::rusage {
        ru_utime: libc::timeval {
            tv_sec: 0 as libc::time_t,
            tv_usec: 0 as libc::suseconds_t,
        },
        ru_stime: libc::timeval {
            tv_sec: 0 as libc::time_t,
            tv_usec: 0 as libc::suseconds_t,
        },
        ru_maxrss: 0 as libc::c_long,
        ru_ixrss: 0 as libc::c_long,
        ru_idrss: 0 as libc::c_long,
        ru_isrss: 0 as libc::c_long,
        ru_minflt: 0 as libc::c_long,
        ru_majflt: 0 as libc::c_long,
        ru_nswap: 0 as libc::c_long,
        ru_inblock: 0 as libc::c_long,
        ru_oublock: 0 as libc::c_long,
        ru_msgsnd: 0 as libc::c_long,
        ru_msgrcv: 0 as libc::c_long,
        ru_nsignals: 0 as libc::c_long,
        ru_nvcsw: 0 as libc::c_long,
        ru_nivcsw: 0 as libc::c_long,
    }
}