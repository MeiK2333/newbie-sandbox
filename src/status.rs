use std::fmt;

#[derive(Debug)]
pub struct RunnerStatus {
    pub time_used: i64,
    pub memory_used: i64,
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