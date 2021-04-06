use std::fs;
use std::fs::{read_to_string, remove_dir};
use std::path::PathBuf;

use libc;
use tempfile::tempdir_in;

use crate::error::Result;

pub struct CGroupOptions {
    pub version: i32,
    /// 允许通过 fork 与 clone 产生的最大进程数量
    pub pids: i32,
}

impl CGroupOptions {}

#[allow(dead_code)]
pub struct CGroup {
    v1: Option<CGroupV1>,
    v2: Option<CGroupV2>,
}

impl CGroup {
    pub fn apply(options: CGroupOptions) -> Result<Self> {
        let pid = unsafe { libc::getpid() };
        let mut v1 = None;
        let mut v2 = None;
        if options.version == 1 {
            v1 = Option::from(CGroupV1::apply(pid, options)?);
        } else if options.version == 2 {
            v2 = Option::from(CGroupV2::apply(pid, options)?);
        }
        Ok(CGroup {
            v1,
            v2,
        })
    }
}

pub struct CGroupV1 {
    pids_path: Option<PathBuf>,
}

impl CGroupV1 {
    pub fn apply(pid: i32, options: CGroupOptions) -> Result<Self> {
        let mut pids_path = None;
        if options.pids > 0 {
            let pwd = try_io!(tempdir_in("/sys/fs/cgroup/pids"));
            pids_path = Some(pwd.path().to_path_buf());
            trace!("cgroup v1 pids path = {:?}", pwd.path());
            try_io!(fs::write(pwd.path().join("cgroup.procs"), format!("{}", pid)));
            try_io!(fs::write(pwd.path().join("pids.max"), format!("{}", options.pids)));
        }

        Ok(CGroupV1 {
            pids_path
        })
    }
}

impl Drop for CGroupV1 {
    fn drop(&mut self) {
        if let Some(path) = &self.pids_path {
            let pids = read_to_string(path.join("cgroup.procs")).unwrap();
            fs::write("/sys/fs/cgroup/pids/cgroup.procs", pids).unwrap();
            remove_dir(path).unwrap();
        }
    }
}


pub struct CGroupV2 {
    path: PathBuf,
}

impl CGroupV2 {
    pub fn apply(pid: i32, options: CGroupOptions) -> Result<Self> {
        // 新建 cgroup v2 目录
        let pwd = try_io!(tempdir_in("/sys/fs/cgroup"));
        trace!("cgroup v2 path = {:?}", pwd.path());
        // 将指定进程加入 cgroup 组里
        try_io!(fs::write(pwd.path().join("cgroup.procs"), format!("{}", pid)));

        if options.pids > 0 {
            try_io!(fs::write(pwd.path().join("pids.max"), format!("{}", options.pids)));
        }

        Ok(CGroupV2 {
            path: pwd.path().to_path_buf()
        })
    }
}

impl Drop for CGroupV2 {
    fn drop(&mut self) {
        // 将当前控制组里所有进程移动到全局 root 节点
        let pids = read_to_string(self.path.join("cgroup.procs")).unwrap();
        fs::write("/sys/fs/cgroup/cgroup.procs", pids).unwrap();
        remove_dir(&self.path).unwrap();
    }
}
