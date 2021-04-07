use std::ptr;

use libc;

use crate::cgroups::{CGroup, CGroupOptions};
use crate::error::Result;
use crate::exec_args::ExecArgs;
use crate::runit;
use crate::runit::wait_it;
use crate::status::RunnerStatus;

const STACK_SIZE: usize = 1024 * 1024;

pub struct Sandbox {
    inner_args: Vec<String>,
    pub workdir: String,
    pub rootfs: String,
    result: Option<String>,
    pub result_fd: i32,
    stdin: Option<String>,
    pub stdin_fd: i32,
    stdout: Option<String>,
    pub stdout_fd: i32,
    stderr: Option<String>,
    pub stderr_fd: i32,
    pub time_limit: Option<i32>,
    pub memory_limit: Option<i32>,
    pub file_size_limit: Option<i32>,
    pub cgroup: i32,
    pub pids: i32,
}

impl Sandbox {
    pub fn new(args: Vec<String>) -> Self {
        // 为什么传 String 而非 &str，因为数据最终会被 unsafe 到子进程中并且主动 forget 与 drop，如果使用 str 会导致生命周期完全混乱
        // 因此使用 String，在传递时复制而非传递地址
        let mut v = vec![String::from("/usr/bin/runit")];
        // let mut v = vec![];
        v.extend(args);
        Sandbox {
            inner_args: v,
            workdir: String::from(""),
            rootfs: String::from(""),
            result: None,
            result_fd: 1,
            stdin: None,
            stdin_fd: 0,
            stdout: None,
            stdout_fd: 1,
            stderr: None,
            stderr_fd: 2,
            time_limit: None,
            memory_limit: None,
            file_size_limit: None,
            cgroup: 1,
            pids: 0,
        }
    }
    // 工作目录，如果没提供则会使用当前目录，始终会被 mount 为沙盒内部的 /tmp
    pub fn workdir(mut self, s: String) -> Self {
        debug!("workdir file = {}", s);
        self.workdir = s.clone();
        self
    }
    pub fn rootfs(mut self, s: String) -> Self {
        self.rootfs = s;
        self
    }
    pub fn result(mut self, s: String) -> Self {
        if s != "/STDOUT/" {
            debug!("result file = {}", s);
            self.result = Some(s.clone());
            self.result_fd = unsafe {
                syscall_or_panic!(libc::open(c_str_ptr!(s), libc::O_CREAT | libc::O_RDWR, 0o644))
            };
        }
        self
    }
    pub fn stdin(mut self, s: String) -> Self {
        if s != "/STDIN/" {
            debug!("stdin file = {}", s);
            self.stdin = Some(s.clone());
            self.stdin_fd = unsafe {
                syscall_or_panic!(libc::open(c_str_ptr!(s), libc::O_RDONLY, 0o644))
            };
        }
        self
    }
    pub fn stdout(mut self, s: String) -> Self {
        if s != "/STDOUT/" {
            debug!("stdout file = {}", s);
            self.stdout = Some(s.clone());
            self.stdout_fd = unsafe {
                syscall_or_panic!(libc::open(c_str_ptr!(s), libc::O_CREAT | libc::O_RDWR, 0o644))
            };
        }
        self
    }
    pub fn stderr(mut self, s: String) -> Self {
        if s != "/STDERR/" {
            debug!("stderr file = {}", s);
            self.stderr = Some(s.clone());
            self.stderr_fd = unsafe {
                syscall_or_panic!(libc::open(c_str_ptr!(s), libc::O_CREAT | libc::O_RDWR, 0o644))
            };
        }
        self
    }
    pub fn time_limit(mut self, l: i32) -> Self {
        if l != 0 {
            self.time_limit = Some(l);
        }
        self
    }
    pub fn memory_limit(mut self, l: i32) -> Self {
        if l != 0 {
            self.memory_limit = Some(l);
        }
        self
    }
    pub fn file_size_limit(mut self, l: i32) -> Self {
        if l != 0 {
            self.file_size_limit = Some(l);
        }
        self
    }
    pub fn cgroup(mut self, l: i32) -> Self {
        if l == 1 || l == 2 {
            self.cgroup = l;
        }
        self
    }
    pub fn pids(mut self, l: i32) -> Self {
        if l > 0 {
            self.pids = l;
        }
        self
    }
    pub fn exec_args(&self) -> Result<ExecArgs> {
        ExecArgs::build(&self.inner_args)
    }
}

impl Sandbox {
    pub fn run(&mut self) -> RunnerStatus {
        let pids = if self.pids > 0 { self.pids + 3 } else { 0 };
        let options = CGroupOptions {
            version: self.cgroup,
            pids,
        };
        let cgroup = CGroup::apply(options).unwrap();
        let stack = unsafe {
            libc::mmap(
                ptr::null_mut(),
                STACK_SIZE,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_STACK,
                -1,
                0,
            )
        };
        if stack == libc::MAP_FAILED {
            let err = std::io::Error::last_os_error().raw_os_error();
            panic!(crate::error::errno_str(err));
        }
        let pid = unsafe {
            syscall_or_panic!(libc::clone(
                runit::runit,
                (stack as usize + STACK_SIZE) as *mut libc::c_void,
                libc::SIGCHLD
                    | libc::CLONE_NEWUTS  // 设置新的 UTS 名称空间（主机名、网络名等）
                    | libc::CLONE_NEWNET  // 设置新的网络空间，如果没有配置网络，则该沙盒内部将无法联网
                    | libc::CLONE_NEWNS  // 为沙盒内部设置新的 namespaces 空间
                    | libc::CLONE_NEWIPC  // IPC 隔离
                    | libc::CLONE_NEWCGROUP  // 在新的 CGROUP 中创建沙盒
                    | libc::CLONE_NEWPID, // 外部进程对沙盒不可见
                self as *mut _ as *mut libc::c_void,
            ))
        };
        debug!("run sandbox pid = {}", pid);
        let status = wait_it(pid);
        unsafe {
            syscall_or_panic!(libc::munmap(stack as *mut libc::c_void, STACK_SIZE));
            drop(cgroup);
        }
        status
    }
}
