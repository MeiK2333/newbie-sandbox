#[macro_use]
extern crate log;

use clap::{ArgSettings, Parser};
use env_logger::Builder;
use log::LevelFilter;

mod utils;
mod error;
mod sandbox;
mod runit;
mod exec_args;
mod status;
mod seccomp;
mod cgroups;

/// example: `newbie-sandbox -- /usr/bin/echo hello world`
#[derive(Parser)]
#[clap(version = "1.0", author = "MeiK <meik2333@gmail.com>")]
struct Opts {
    /// 输入流，默认为 STDIN(0)
    #[clap(short, long, default_value = "/STDIN/")]
    input: String,
    /// 输出流，默认为 STDOUT(1)
    #[clap(short, long, default_value = "/STDOUT/")]
    output: String,
    /// 错误流，默认为 STDERR(2)
    #[clap(short, long, default_value = "/STDERR/")]
    error: String,
    /// 工作目录，默认为当前目录
    #[clap(short, long, default_value = "./")]
    workdir: String,
    /// 沙盒所需的运行文件，必须存在
    #[clap(long, default_value = "./runtime/rootfs")]
    rootfs: String,
    /// 运行结果输出位置，默认为 STDOUT(1)
    #[clap(short, long, default_value = "/STDOUT/")]
    result: String,
    /// 运行 CPU 时间限制，单位 ms，默认无限制
    #[clap(short, long, default_value = "0")]
    time_limit: i32,
    /// 运行内存限制，单位 kib，默认无限制
    #[clap(short, long, default_value = "0")]
    memory_limit: i32,
    /// 可写入的文件限制，单位 bit，默认无限制
    #[clap(short, long, default_value = "0")]
    file_size_limit: i32,
    /// cgroup 版本，1 或 2
    #[clap(short, long, default_value = "1")]
    cgroup: i32,
    /// 最大可创建的 pid 数量，默认无限制
    #[clap(short, long, default_value = "0")]
    pids: i32,
    /// 要运行的程序及命令行参数
    #[clap(setting = ArgSettings::Last, required = true)]
    command: Vec<String>,
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
}

fn main() {
    let opts: Opts = Opts::parse();

    let log_level = match opts.verbose {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        3 | _ => LevelFilter::Trace,
    };
    Builder::new().filter_level(log_level).init();

    let status = sandbox::Sandbox::new(opts.command)
        .rootfs(opts.rootfs)
        .stdin(opts.input)
        .stdout(opts.output)
        .stderr(opts.error)
        .time_limit(opts.time_limit)
        .memory_limit(opts.memory_limit)
        .file_size_limit(opts.file_size_limit)
        .cgroup(opts.cgroup)
        .pids(opts.pids)
        .workdir(opts.workdir)
        .result(opts.result)
        .run();


    // 此处获取的数值为沙盒的总资源用量，我们指定进程的资源占用需要用 result 获取
    debug!("sandbox time used   = {}", status.time_used);
    debug!("sandbox memory used = {}", status.memory_used);
    debug!("sandbox exit_code   = {}", status.exit_code);
    debug!("sandbox status      = {}", status.status);
    debug!("sandbox signal      = {}", status.signal);
}
