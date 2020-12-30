use libc;
use std::env;
use std::ffi::CStr;
use std::ffi::CString;
use std::io;
use std::mem;
use std::os::raw::c_char;
use std::ptr;
use std::slice;

const STACK_SIZE: usize = 1024 * 1024;

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
        | libc::CLONE_NEWUSER  // 为沙盒内部选择新的 User
        | libc::CLONE_NEWIPC  // IPC 隔离
        | libc::CLONE_NEWCGROUP  // 在新的 CGROUP 中创建沙盒
        | libc::CLONE_NEWPID, // 外部进程对沙盒不可见
      env_args() as *mut libc::c_void,
    );
    if pid < 0 {
      let err = io::Error::last_os_error().raw_os_error();
      println!("clone error: {}", errno_str(err));
    } else {
      println!("newbie-sandbox pid = {}", pid);
    }
    let mut status: i32 = 0;
    libc::wait4(pid, &mut status, 0, ptr::null_mut());

    libc::munmap(stack, STACK_SIZE);
  }
}

extern "C" fn process(args: *mut libc::c_void) -> i32 {
  unsafe {
    // 设置主机名
    libc::sethostname(CString::new("nb_sandbox").unwrap().as_ptr(), 10);
    libc::setdomainname(CString::new("nb_sandbox").unwrap().as_ptr(), 10);

    // 挂载文件
    libc::mount(
      CString::new("proc").unwrap().as_ptr(),
      CString::new("rootfs/proc").unwrap().as_ptr(),
      CString::new("proc").unwrap().as_ptr(),
      0,
      ptr::null_mut(),
    );

    // chdir && chroot，隔离文件系统
    libc::chdir(CString::new("rootfs").unwrap().as_ptr());
    libc::chroot(CString::new("./").unwrap().as_ptr());

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
