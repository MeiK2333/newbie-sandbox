use std::convert::TryInto;
use std::ptr;
use std::{thread, time};

use crate::sandbox::Sandbox;
use crate::seccomp;
use crate::status::RunnerStatus;
use crate::utils;

extern "C" fn timer_thread(sandbox: *mut libc::c_void) -> *mut libc::c_void {
    trace!("timer thread");
    let sandbox = unsafe { &mut *(sandbox as *mut Sandbox) };

    if let Some(time_limit) = sandbox.time_limit {
        trace!("time limit = {}", time_limit);
        thread::sleep(time::Duration::from_secs((time_limit / 1000 + 2) as u64));
        unsafe {
            killpid(3);
        }
        trace!("timer thread done");
    }
    ptr::null_mut()
}

pub extern "C" fn runit(sandbox: *mut libc::c_void) -> i32 {
    let sandbox = unsafe { &mut *(sandbox as *mut Sandbox) };
    let exec_args = sandbox.exec_args().unwrap();

    let pid = unsafe { syscall_or_panic!(libc::fork()) };
    // 当前进程（沙盒内部 pid = 1）
    if pid > 0 {
        if pid != 2 {
            panic!("System Error!");
        }
        // 等待进程结束之后，我们才能继续等待 3 这个进程
        // 因为在 3 的父进程没退出的时候，3 这个进程还是归 2 所有的，只有 2 退出后，3 才会作为孤儿进程被 1 接管
        let _status = wait_it(pid);

        // 创建一个新线程来监听真实时间
        let mut timer_thread_id = 0;
        if let Some(_) = sandbox.time_limit {
            unsafe {
                libc::pthread_create(
                    &mut timer_thread_id,
                    ptr::null_mut(),
                    timer_thread,
                    sandbox as *mut _ as *mut libc::c_void,
                );
            }
        }

        // 得益于 Linux 的设计，我们可以使用当前进程（pid = 1）wait 沙盒内部任意孤儿进程
        // 通过三次跳转，我们能够排除掉大部分中间的影响因素，从而获取最接近准确的测量结果（代价是三个额外的进程）
        // 如果因系统异常，3 进程在 2 进程退出前就退出了，那么此处 wait 将会失败，常见原因是资源限制过小，导致无法获取运行必需的资源
        let status = wait_it(3);

        // 在进程结束后取消线程
        if timer_thread_id != 0 {
            unsafe {
                libc::pthread_cancel(timer_thread_id);
            }
        }

        // 此处获取的数值即为我们最终结果的数值
        debug!("time used   = {}", status.time_used);
        debug!("memory used = {}", status.memory_used);
        debug!("exit_code   = {}", status.exit_code);
        debug!("status      = {}", status.status);
        debug!("signal      = {}", status.signal);
        status.result_to_fd(sandbox.result_fd).unwrap();
        return 0;
    }

    // 子进程（pid = 2）
    unsafe {
        // 资源限制（使用 setrlimit）
        let mut rlimit = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        // CPU 时间限制，单位为 S
        if let Some(time_limit) = sandbox.time_limit {
            rlimit.rlim_cur = (time_limit / 1000 + 1) as u64;
            if time_limit % 1000 > 800 {
                rlimit.rlim_cur += 1;
            }
            rlimit.rlim_max = rlimit.rlim_cur;
            syscall_or_panic!(libc::setrlimit(libc::RLIMIT_CPU, &rlimit));
        }
        // 内存限制，单位为 kib
        if let Some(memory_limit) = sandbox.memory_limit {
            rlimit.rlim_cur = memory_limit as u64 * 1024 * 2;
            rlimit.rlim_max = memory_limit as u64 * 1024 * 2;
            syscall_or_panic!(libc::setrlimit(libc::RLIMIT_AS, &rlimit));
        }
        // 文件大小限制，单位为 bit
        if let Some(file_size_limit) = sandbox.file_size_limit {
            rlimit.rlim_cur = file_size_limit as u64;
            rlimit.rlim_max = file_size_limit as u64;
            syscall_or_panic!(libc::setrlimit(libc::RLIMIT_FSIZE, &rlimit));
        }

        // 安全机制
        security(&sandbox);

        // 重定向描述符
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
    let _ret = unsafe { syscall_or_panic!(libc::wait4(pid, &mut status, 0, &mut rusage)) };
    let time_used = rusage.ru_utime.tv_sec * 1000
        + i64::from(rusage.ru_utime.tv_usec) / 1000
        + rusage.ru_stime.tv_sec * 1000
        + i64::from(rusage.ru_stime.tv_usec) / 1000;
    let memory_used = rusage.ru_maxrss;
    let mut exit_code = 0;
    let exited = libc::WIFEXITED(status);
    if exited {
        exit_code = libc::WEXITSTATUS(status);
    }
    let signal = if libc::WIFSIGNALED(status) {
        libc::WTERMSIG(status)
    } else if libc::WIFSTOPPED(status) {
        libc::WSTOPSIG(status)
    } else {
        0
    };

    RunnerStatus {
        time_used,
        memory_used,
        exit_code,
        signal,
        status,
    }
}

unsafe fn security(sandbox: &Sandbox) {
    // 全局默认权限 755，为运行目录设置特权
    // 因为将会使用 nobody 用户来执行程序，如果没有运行目录 777 权限，将会无法正常工作
    trace!("chmod {} 777", sandbox.workdir);
    syscall_or_panic!(libc::chmod(c_str_ptr!(sandbox.workdir.clone()), 0o777,));
    // 等同于 mount --make-rprivate /
    // 不将挂载传播到其他空间，以免造成挂载混淆
    syscall_or_panic!(libc::mount(
        c_str_ptr!(""),
        c_str_ptr!("/"),
        c_str_ptr!(""),
        libc::MS_PRIVATE | libc::MS_REC,
        ptr::null_mut()
    ));

    // 挂载 /proc 目录，有些语言（比如 rust）依赖此目录
    syscall_or_panic!(libc::mount(
        c_str_ptr!("proc"),
        c_str_ptr!(format!("{}/proc", sandbox.rootfs)),
        c_str_ptr!("proc"),
        0,
        ptr::null_mut(),
    ));

    // 挂载运行文件夹，除此目录外程序没有其他目录的写权限
    syscall_or_panic!(libc::mount(
        c_str_ptr!(sandbox.workdir.clone()),
        c_str_ptr!(format!("{}/tmp", sandbox.rootfs)),
        c_str_ptr!("none"),
        libc::MS_BIND | libc::MS_PRIVATE,
        ptr::null_mut(),
    ));

    // chdir && chroot，隔离文件系统
    syscall_or_panic!(libc::chdir(c_str_ptr!(sandbox.rootfs.clone())));
    syscall_or_panic!(libc::chroot(c_str_ptr!(".")));
    syscall_or_panic!(libc::chdir(c_str_ptr!("/tmp")));

    // 设置主机名
    syscall_or_panic!(libc::sethostname(c_str_ptr!("newbie-sandbox"), 14));
    syscall_or_panic!(libc::setdomainname(c_str_ptr!("newbie-sandbox"), 14));

    // 修改用户为 nobody
    syscall_or_panic!(libc::setgid(65534));
    syscall_or_panic!(libc::setuid(65534));

    let filter = seccomp::SeccompFilter::new(
        deny_syscalls().into_iter().collect(),
        seccomp::SeccompAction::Allow,
    )
    .unwrap();
    seccomp::SeccompFilter::apply(filter.try_into().unwrap()).unwrap();
}

/// 阻止危险的系统调用
///
/// 参照 Docker 文档 [significant-syscalls-blocked-by-the-default-profile](https://docs.docker.com/engine/security/seccomp/#significant-syscalls-blocked-by-the-default-profile) 一节
fn deny_syscalls() -> Vec<seccomp::SyscallRuleSet> {
    vec![
        deny_syscall(libc::SYS_acct),
        deny_syscall(libc::SYS_add_key),
        deny_syscall(libc::SYS_bpf),
        deny_syscall(libc::SYS_clock_adjtime),
        deny_syscall(libc::SYS_clock_settime),
        deny_syscall(libc::SYS_create_module),
        deny_syscall(libc::SYS_delete_module),
        deny_syscall(libc::SYS_finit_module),
        deny_syscall(libc::SYS_get_kernel_syms),
        deny_syscall(libc::SYS_get_mempolicy),
        deny_syscall(libc::SYS_init_module),
        deny_syscall(libc::SYS_ioperm),
        deny_syscall(libc::SYS_iopl),
        deny_syscall(libc::SYS_kcmp),
        deny_syscall(libc::SYS_kexec_file_load),
        deny_syscall(libc::SYS_kexec_load),
        deny_syscall(libc::SYS_keyctl),
        deny_syscall(libc::SYS_lookup_dcookie),
        deny_syscall(libc::SYS_mbind),
        deny_syscall(libc::SYS_mount),
        deny_syscall(libc::SYS_move_pages),
        deny_syscall(libc::SYS_name_to_handle_at),
        deny_syscall(libc::SYS_nfsservctl),
        deny_syscall(libc::SYS_open_by_handle_at),
        deny_syscall(libc::SYS_perf_event_open),
        deny_syscall(libc::SYS_personality),
        deny_syscall(libc::SYS_pivot_root),
        deny_syscall(libc::SYS_process_vm_readv),
        deny_syscall(libc::SYS_process_vm_writev),
        deny_syscall(libc::SYS_ptrace),
        deny_syscall(libc::SYS_query_module),
        deny_syscall(libc::SYS_quotactl),
        deny_syscall(libc::SYS_reboot),
        deny_syscall(libc::SYS_request_key),
        deny_syscall(libc::SYS_set_mempolicy),
        deny_syscall(libc::SYS_setns),
        deny_syscall(libc::SYS_settimeofday),
        deny_syscall(libc::SYS_swapon),
        deny_syscall(libc::SYS_swapoff),
        deny_syscall(libc::SYS_sysfs),
        deny_syscall(libc::SYS__sysctl),
        deny_syscall(libc::SYS_umount2),
        deny_syscall(libc::SYS_unshare),
        deny_syscall(libc::SYS_uselib),
        deny_syscall(libc::SYS_userfaultfd),
        deny_syscall(libc::SYS_ustat),
    ]
}

#[inline(always)]
fn deny_syscall(syscall_number: i64) -> seccomp::SyscallRuleSet {
    (
        syscall_number,
        vec![seccomp::SeccompRule::new(
            vec![],
            seccomp::SeccompAction::Kill,
        )],
    )
}

unsafe fn killpid(pid: i32) {
    let mut status = 0;

    trace!("kill pid {} if exists", pid);
    // > 0: 对应子进程退出但未回收资源
    // = 0: 对应子进程存在但未退出
    // 如果在运行过程中上层异常中断，则需要 kill 子进程并回收资源
    if libc::waitpid(pid, &mut status, libc::WNOHANG) >= 0 {
        trace!("kill pid {}", pid);
        libc::kill(pid, 9);
        libc::waitpid(pid, &mut status, 0);
    }
}
