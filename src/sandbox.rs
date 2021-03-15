use crate::error::Result;
use crate::exec_args::ExecArgs;
use crate::runit;
use libc;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::ptr;
use std::sync::{mpsc, Arc, Mutex};
use std::task::{Context, Poll};
use std::thread;

#[cfg(test)]
use std::println as debug;

struct Stack {
    stack: *const libc::c_void,
}

unsafe impl Send for Stack {}
unsafe impl Sync for Stack {}

const STACK_SIZE: usize = 1024 * 1024;

pub struct Sandbox {
    inner_args: Vec<String>,
}

#[allow(dead_code)]
impl Sandbox {
    pub fn new(args: Vec<String>) -> Self {
        // 为什么传 String 而非 &str，因为数据最终会被 unsafe 到子进程中并且主动 forget 与 drop，如果使用 str 会导致生命周期完全混乱
        // 因此使用 String，在传递时复制而非传递地址
        // let mut v = vec![String::from("/bin/runit")];
        let mut v = vec![];
        v.extend(args);
        debug!("sandbox args = {:?}", v);
        Sandbox { inner_args: v }
    }
    // 工作目录，如果没提供则会自动创建一个，始终会被 mount 为沙盒内部的 /tmp
    pub fn workdir() {}
    pub fn stdin() {}
    pub fn stdout() {}
    pub fn stderr() {}
    pub fn time_limit() {}
    pub fn memory_limit() {}
    pub fn exec_args(&self) -> Result<ExecArgs> {
        ExecArgs::build(&self.inner_args.to_vec())
    }
}

pub struct Runner {
    process: Sandbox,
    pid: i32,
    tx: Arc<Mutex<mpsc::Sender<RunnerStatus>>>,
    rx: Arc<Mutex<mpsc::Receiver<RunnerStatus>>>,
    stack: Stack,
}

#[allow(dead_code)]
impl Runner {
    pub fn from(process: Sandbox) -> Runner {
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
        let (tx, rx) = mpsc::channel();
        let runner = Runner {
            process,
            pid: -1,
            tx: Arc::new(Mutex::new(tx)),
            rx: Arc::new(Mutex::new(rx)),
            stack: Stack { stack },
        };
        runner
    }
}

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

impl Future for Runner {
    type Output = Result<RunnerStatus>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<RunnerStatus>> {
        let runner = Pin::into_inner(self);
        if runner.pid == -1 {
            let pid = unsafe {
                libc::clone(
                    runit::runit,
                    (runner.stack.stack as usize + STACK_SIZE) as *mut libc::c_void,
                    libc::SIGCHLD
                    | libc::CLONE_NEWUTS  // 设置新的 UTS 名称空间（主机名、网络名等）
                    | libc::CLONE_NEWNET  // 设置新的网络空间，如果没有配置网络，则该沙盒内部将无法联网
                    | libc::CLONE_NEWNS  // 为沙盒内部设置新的 namespaces 空间
                    | libc::CLONE_NEWIPC  // IPC 隔离
                    | libc::CLONE_NEWCGROUP  // 在新的 CGROUP 中创建沙盒
                    | libc::CLONE_NEWPID, // 外部进程对沙盒不可见
                    &mut runner.process as *mut _ as *mut libc::c_void,
                )
            };
            let waker = cx.waker().clone();
            let tx = runner.tx.clone();
            runner.pid = pid;
            thread::spawn(move || {
                let status = runit::wait_it(pid);
                let status_tx = tx.lock().unwrap();
                status_tx.send(status).unwrap();
                waker.wake();
            });
            return Poll::Pending;
        }
        let status = runner.rx.lock().unwrap().recv().unwrap();
        Poll::Ready(Ok(status))
    }
}

unsafe fn kill_pid(pid: i32) {
    let mut status = 0;
    // > 0: 对应子进程退出但未回收资源
    // = 0: 对应子进程存在但未退出
    // 如果在运行过程中上层异常中断，则需要 kill 子进程并回收资源
    if libc::waitpid(pid, &mut status, libc::WNOHANG) >= 0 {
        libc::kill(pid, 9);
        libc::waitpid(pid, &mut status, 0);
    }
}

impl Drop for Runner {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.stack.stack as *mut libc::c_void, STACK_SIZE);
            if self.pid > 0 {
                kill_pid(self.pid);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试基本流程
    #[tokio::test]
    async fn test_echo() {
        let sandbox = Sandbox::new(vec![
            String::from("/bin/echo"),
            String::from("Hello"),
            String::from("World"),
        ]);
        let runner = Runner::from(sandbox);
        let _ = runner.await;
    }
}
