use libc;
use std::convert::TryInto;
use std::env;
use std::ffi::CStr;
use std::ffi::CString;
use std::fs;
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::mem;
use std::os::raw::c_char;
use std::ptr;
use std::slice;
use std::time::SystemTime;

mod seccomp;

const STACK_SIZE: usize = 1024 * 1024;

macro_rules! c_str_ptr {
    ($expression:expr) => {
        CString::new($expression).unwrap().as_ptr()
    };
}

fn main() {
    let stack;
    unsafe {
        stack = libc::mmap(
            ptr::null_mut(),
            STACK_SIZE,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_STACK,
            -1,
            0,
        );
        let pid = libc::clone(
            process,
            (stack as usize + STACK_SIZE) as *mut libc::c_void,
            libc::SIGCHLD
            | libc::CLONE_NEWUTS  // 设置新的 UTS 名称空间（主机名、网络名等）
            | libc::CLONE_NEWNET  // 设置新的网络空间，如果没有配置网络，则该沙盒内部将无法联网
            | libc::CLONE_NEWNS  // 为沙盒内部设置新的 namespaces 空间
            // | libc::CLONE_NEWUSER  // 为沙盒内部选择新的 User
            | libc::CLONE_NEWIPC  // IPC 隔离
            | libc::CLONE_NEWCGROUP  // 在新的 CGROUP 中创建沙盒
            | libc::CLONE_NEWPID, // 外部进程对沙盒不可见
            env_args() as *mut libc::c_void,
        );
        if pid < 0 {
            let err = io::Error::last_os_error().raw_os_error();
            println!("clone error: {}", errno_str(err));
            return;
        }
        println!("newbie-sandbox pid = {}", pid);
        // 创建 cgroup 组并将沙盒进程加入
        let cgroup = format!("/sys/fs/cgroup/nb_sandbox-{}", pid);
        fs::create_dir(&cgroup).unwrap();
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(format!("{}/cgroup.procs", &cgroup))
            .unwrap();
        let _ = writeln!(file, "{}", pid);

        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(format!("{}/memory.max", &cgroup))
            .unwrap();
        // 设置 64m 内存限制
        let _ = writeln!(file, "67108864");

        let mut status: i32 = 0;
        let mut rusage = libc::rusage {
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
        };
        let start = SystemTime::now();
        libc::wait4(pid, &mut status, 0, &mut rusage);
        let time_used = rusage.ru_utime.tv_sec * 1000
            + i64::from(rusage.ru_utime.tv_usec) / 1000
            + rusage.ru_stime.tv_sec * 1000
            + i64::from(rusage.ru_stime.tv_usec) / 1000;
        let memory_used = rusage.ru_maxrss;
        // 此处获取的内存用量为进程退出后的，大概率为 0，没有参考价值
        // 如果想获取比较有意义的值，可以考虑进程运行中多次中断来读取内存占用，不过这样对性能会有一定影响
        // 而且需要 ptrace 以中断进程，会导致某些程序无法执行（比如 gcc 等）
        // TODO 此处需要一个解决方案
        let cgroup_memory_used = fs::read_to_string(format!("{}/memory.current", &cgroup)).unwrap();
        let real_time_used = start.elapsed().unwrap().as_millis();
        println!("cpu time used:      {}", time_used);
        println!("real time used:     {}", real_time_used);
        println!("memory used:        {}", memory_used);
        println!("cgroup memory used: {}", cgroup_memory_used);

        libc::munmap(stack, STACK_SIZE);
        fs::remove_dir(&cgroup).unwrap();
    }
}

extern "C" fn process(args: *mut libc::c_void) -> i32 {
    unsafe {
        // 设置主机名
        libc::sethostname(c_str_ptr!("nb_sandbox"), 10);
        libc::setdomainname(c_str_ptr!("nb_sandbox"), 10);

        // 挂载文件
        libc::mount(
            c_str_ptr!("proc"),
            c_str_ptr!("rootfs/proc"),
            c_str_ptr!("proc"),
            0,
            ptr::null_mut(),
        );

        // chdir && chroot，隔离文件系统
        libc::chdir(c_str_ptr!("rootfs"));
        libc::chroot(c_str_ptr!("./"));

        // 修改用户为 nobody
        libc::setgid(65534);
        libc::setuid(65534);

        let filter = seccomp::SeccompFilter::new(
            gen_rules().into_iter().collect(),
            seccomp::SeccompAction::Allow,
        )
        .unwrap();
        seccomp::SeccompFilter::apply(filter.try_into().unwrap()).unwrap();

        let args = args as *const *const c_char;
        let slice = slice::from_raw_parts(args, 1);

        libc::execve(
            slice[0] as *const c_char,
            args as *const *const i8,
            ptr::null_mut(),
        );
        let err = io::Error::last_os_error().raw_os_error();
        println!("execve error: {}", errno_str(err));
    }
    return 0;
}

fn errno_str(errno: Option<i32>) -> String {
    match errno {
        Some(no) => {
            let stre = unsafe { libc::strerror(no) };
            let c_str: &CStr = unsafe { CStr::from_ptr(stre) };
            c_str.to_str().unwrap().to_string()
        }
        _ => "Unknown Error!".to_string(),
    }
}

// WARN: 此处没有释放内存，有内存泄露
fn env_args() -> *const *const libc::c_char {
    let args: Vec<String> = env::args().collect();
    let mut argv_vec: Vec<*const libc::c_char> = vec![];
    for item in args.iter().skip(1) {
        let cstr = CString::new(item.clone()).unwrap();
        let cptr = cstr.as_ptr();
        // 需要使用 mem::forget 来标记
        // 否则在此次循环结束后，cstr 就会被回收，后续 exec 函数无法通过指针获取到字符串内容
        mem::forget(cstr);
        argv_vec.push(cptr);
    }
    // argv 的参数需要使用 NULL 来标记结束
    argv_vec.push(ptr::null());
    let argv: *const *const libc::c_char = argv_vec.as_ptr() as *const *const libc::c_char;
    mem::forget(argv_vec);
    argv
}

fn gen_rules() -> Vec<seccomp::SyscallRuleSet> {
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
