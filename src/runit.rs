use crate::sandbox::Sandbox;
use crate::status::RunnerStatus;
use crate::utils;

pub extern "C" fn runit(sandbox: *mut libc::c_void) -> i32 {
    let sandbox = unsafe { &mut *(sandbox as *mut Sandbox) };
    let exec_args = sandbox.exec_args().unwrap();

    let pid = unsafe {
        syscall_or_panic!(libc::fork())
    };
    // 当前进程（沙盒内部 pid = 1）
    if pid > 0 {
        let _status = wait_it(pid);
        // TODO: 使用 runit.s 继续 fork & exec，此处需要 wait pid = 3 的进程（沙盒内部 pid 从 1 开始，1 为当前进程，2 是子进程 & runit.s，3 是我们要执行的目标进程）
        // 得益于 Linux 的设计，我们可以使用当前进程（pid = 1）wait 沙盒内部任意孤儿进程
        // 通过三次跳转，我们能够排除掉大部分中间的影响因素，从而获取最接近准确的测量结果（代价是三个额外的进程）
        // wait_it(3)
        return 0;
    }

    // 子进程（pid = 2）
    unsafe {
        // TODO: 安全机制、资源限制等
        syscall_or_panic!(libc::dup2(sandbox.stdin_fd, libc::STDIN_FILENO));
        syscall_or_panic!(libc::dup2(sandbox.stdout_fd, libc::STDOUT_FILENO));
        syscall_or_panic!(libc::dup2(sandbox.stderr_fd, libc::STDERR_FILENO));
    }

    let ret = unsafe {
        syscall_or_panic!(libc::execve(
            exec_args.pathname,
            exec_args.argv,
            exec_args.envp
        ))
    };
    // 实际这个函数不会返回了，exec 之后会直接被新程序替换掉
    ret
}

pub fn wait_it(pid: i32) -> RunnerStatus {
    let mut status: i32 = 0;
    let mut rusage = utils::new_rusage();
    let _ret = unsafe {
        syscall_or_panic!(libc::wait4(pid, &mut status, 0, &mut rusage))
    };
    let time_used = rusage.ru_utime.tv_sec * 1000
        + i64::from(rusage.ru_utime.tv_usec) / 1000
        + rusage.ru_stime.tv_sec * 1000
        + i64::from(rusage.ru_stime.tv_usec) / 1000;
    let memory_used = rusage.ru_maxrss;

    RunnerStatus {
        time_used,
        memory_used,
    }
}
