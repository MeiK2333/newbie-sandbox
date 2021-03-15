use crate::sandbox::RunnerStatus;
use crate::sandbox::Sandbox;
use crate::utils;

#[cfg(test)]
use std::println as debug;

pub extern "C" fn runit(sandbox: *mut libc::c_void) -> i32 {
    let sandbox = unsafe { &mut *(sandbox as *mut Sandbox) };
    let exec_args = sandbox.exec_args().unwrap();
    unsafe {
        syscall_or_panic!(libc::execve(
            exec_args.pathname,
            exec_args.argv,
            exec_args.envp
        ));
    }
    // 实际这个函数不会返回了，exec 之后会直接被新程序替换掉
    0
}

pub fn wait_it(pid: i32) -> RunnerStatus {
    debug!("wait pid = {}", pid);
    let mut status: i32 = 0;
    let mut rusage = utils::new_rusage();
    unsafe {
        libc::wait4(pid, &mut status, 0, &mut rusage);
    }
    let time_used = rusage.ru_utime.tv_sec * 1000
        + i64::from(rusage.ru_utime.tv_usec) / 1000
        + rusage.ru_stime.tv_sec * 1000
        + i64::from(rusage.ru_stime.tv_usec) / 1000;
    let memory_used = rusage.ru_maxrss;
    debug!("cpu time used:      {}", time_used);
    debug!("memory used:        {}", memory_used);
    RunnerStatus {
        time_used,
        memory_used,
    }
}
