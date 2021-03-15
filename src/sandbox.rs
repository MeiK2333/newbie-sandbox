use std::ptr;

use libc;

use crate::error::Result;
use crate::exec_args::ExecArgs;
use crate::runit;
use crate::runit::wait_it;
use crate::status::RunnerStatus;

const STACK_SIZE: usize = 1024 * 1024;

pub struct Sandbox {
    inner_args: Vec<String>,
    workdir: Option<String>,
    rootfs: String,
    result: Option<String>,
    stdin: Option<String>,
    stdout: Option<String>,
    stderr: Option<String>,
    time_limit: Option<i32>,
    memory_limit: Option<i32>,
}

impl Sandbox {
    pub fn new(args: Vec<String>) -> Self {
        // 为什么传 String 而非 &str，因为数据最终会被 unsafe 到子进程中并且主动 forget 与 drop，如果使用 str 会导致生命周期完全混乱
        // 因此使用 String，在传递时复制而非传递地址
        // let mut v = vec![String::from("/bin/runit")];
        let mut v = vec![];
        v.extend(args);
        Sandbox {
            inner_args: v,
            workdir: None,
            rootfs: String::from(""),
            result: None,
            stdin: None,
            stdout: None,
            stderr: None,
            time_limit: None,
            memory_limit: None,
        }
    }
    // 工作目录，如果没提供则会使用当前目录，始终会被 mount 为沙盒内部的 /tmp
    pub fn workdir(mut self, s: String) -> Self {
        if s != "/WORKDIR/" {
            self.workdir = Some(s);
        }
        self
    }
    pub fn rootfs(mut self, s: String) -> Self {
        self.rootfs = s;
        self
    }
    pub fn result(mut self, s: String) -> Self {
        if s != "/STDOUT/" {
            self.result = Some(s);
        }
        self
    }
    pub fn stdin(mut self, s: String) -> Self {
        if s != "/STDIN/" {
            self.stdin = Some(s);
        }
        self
    }
    pub fn stdout(mut self, s: String) -> Self {
        if s != "/STDOUT/" {
            self.stdout = Some(s);
        }
        self
    }
    pub fn stderr(mut self, s: String) -> Self {
        if s != "/STDERR/" {
            self.stderr = Some(s);
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
    pub fn exec_args(&self) -> Result<ExecArgs> {
        ExecArgs::build(&self.inner_args)
    }
}

impl Sandbox {
    pub fn run(&mut self) -> RunnerStatus {
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
        let pid = unsafe {
            libc::clone(
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
            )
        };
        let status = wait_it(pid);
        unsafe {
            libc::munmap(stack as *mut libc::c_void, STACK_SIZE);
        }
        status
    }
}
