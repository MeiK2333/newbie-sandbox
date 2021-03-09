#[macro_use]
extern crate log;

use std::env;

mod utils;

mod error;
mod exec_args;
mod runit;
mod sandbox;
mod seccomp;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let process = sandbox::Sandbox::new(args);
    let runner = sandbox::Runner::from(process);
    let state = runner.await.unwrap();
    println!("{}", state);
}
