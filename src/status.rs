use std::{
    fs::File,
    io::Write,
    os::unix::io::FromRawFd,
};
use std::fmt;

use crate::error::Result;

#[derive(Debug)]
pub struct RunnerStatus {
    pub time_used: i64,
    pub memory_used: i64,
    pub exit_code: i32,
    pub status: i32,
    pub signal: i32,
}

impl fmt::Display for RunnerStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "time_used = {}\nmemory_used = {}",
            self.time_used, self.memory_used
        )
    }
}

impl RunnerStatus {
    pub fn result_to_fd(&self, fd: i32) -> Result<()> {
        let mut f = unsafe { File::from_raw_fd(fd) };
        try_io!(write!(&mut f,
"time_used = {}
memory_used = {}
exit_code = {}
status = {}
signal = {}
", self.time_used, self.memory_used, self.exit_code, self.status, self.signal
        ));
        Ok(())
    }
}
