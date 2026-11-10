extern crate alloc;
mod forwarding;
mod process;
use cargo_prompt::Failure;
use error_stack::Report;
use std::{env, process::ExitCode};
fn execute() -> Result<ExitCode, Report<Failure>> {
    let mut incoming: Vec<_> = env::args_os().skip(1).collect();
    if incoming.first().is_some_and(|arg| arg == "prompt") {
        incoming.remove(0);
    }
    let invocation = forwarding::invocation(incoming)?;
    process::run(&invocation)
}
fn main() -> ExitCode {
    match execute() {
        Ok(status) => status,
        Err(error) => {
            eprintln!("error: {error:?}");
            ExitCode::FAILURE
        }
    }
}
