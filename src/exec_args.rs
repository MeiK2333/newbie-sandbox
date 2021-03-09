use crate::error::Result;
use libc;
use std::collections::HashMap;
use std::ffi::CString;
use std::mem;
use std::ptr;

#[cfg(test)]
use std::println as debug;

pub struct ExecArgs {
    pub pathname: *const libc::c_char,
    pub argv: *const *const libc::c_char,
    pub envp: *const *const libc::c_char,
    args: usize,
    envs: usize,
}

impl ExecArgs {
    pub fn build(args: &Vec<String>) -> Result<ExecArgs> {
        debug!("args = {:?}", args);
        let pathname = args[0].clone();
        let pathname_str = try_cstr!(pathname);
        let pathname = pathname_str.as_ptr();

        let mut argv_vec: Vec<*const libc::c_char> = vec![];
        for item in args.iter() {
            let cstr = try_cstr!(item.clone());
            let cptr = cstr.as_ptr();
            // 需要使用 mem::forget 来标记
            // 否则在此次循环结束后，cstr 就会被回收，后续 exec 函数无法通过指针获取到字符串内容
            mem::forget(cstr);
            argv_vec.push(cptr);
        }
        // argv 与 envp 的参数需要使用 NULL 来标记结束
        argv_vec.push(ptr::null());
        let argv: *const *const libc::c_char = argv_vec.as_ptr() as *const *const libc::c_char;

        // env 传递环境变量
        let mut envs: HashMap<&str, &str> = HashMap::new();
        envs.insert(
            "PATH",
            "/root/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
        );
        envs.insert("HOME", "/tmp");
        envs.insert("TERM", "xterm");
        let mut envp_vec: Vec<*const libc::c_char> = vec![];
        for (key, value) in envs {
            let mut key = String::from(key);
            key.push_str("=");
            key.push_str(&value);
            let cstr = try_cstr!(key);
            let cptr = cstr.as_ptr();
            // 需要使用 mem::forget 来标记
            // 否则在此次循环结束后，cstr 就会被回收，后续 exec 函数无法通过指针获取到字符串内容
            mem::forget(cstr);
            envp_vec.push(cptr);
        }
        envp_vec.push(ptr::null());
        let envs = envp_vec.len();
        let envp = envp_vec.as_ptr() as *const *const libc::c_char;

        mem::forget(pathname_str);
        mem::forget(argv_vec);
        mem::forget(envp_vec);
        Ok(ExecArgs {
            pathname,
            argv,
            args: args.len(),
            envp,
            envs,
        })
    }
}

impl Drop for ExecArgs {
    fn drop(&mut self) {
        // 将 forget 的内存重新获取，并释放
        let c_string = unsafe { CString::from_raw(self.pathname as *mut i8) };
        drop(c_string);
        let argv = unsafe {
            Vec::from_raw_parts(
                self.argv as *mut *const libc::c_void,
                self.args - 1,
                self.args - 1,
            )
        };
        for arg in &argv {
            let c_string = unsafe { CString::from_raw(*arg as *mut i8) };
            drop(c_string);
        }
        drop(argv);
        let envp = unsafe {
            Vec::from_raw_parts(
                self.envp as *mut *const libc::c_void,
                self.envs - 1,
                self.envs - 1,
            )
        };
        for env in &envp {
            let c_string = unsafe { CString::from_raw(*env as *mut i8) };
            drop(c_string);
        }
        drop(envp);
        debug!("DROP");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试 exec args 的生成与释放内存
    #[tokio::test]
    async fn test_exec_args() {
        let _exec_args = ExecArgs::build(&vec![
            String::from("/bin/echo"),
            String::from("Hello"),
            String::from("World"),
        ]);
    }
}
