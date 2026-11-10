use crate::forwarding::Invocation;
use cargo_prompt::{Failure, format};
use error_stack::{Report, ResultExt as _};
use std::{
    env,
    ffi::{OsStr, OsString},
    io::{self, BufReader},
    process::{Command, ExitCode, Stdio},
};
fn run_formatter(cargo: &OsStr, invocation: &Invocation) -> Result<(), Report<Failure>> {
    let status = Command::new(cargo)
        .arg("fmt")
        .args(&invocation.format_arguments)
        .envs(&invocation.environment)
        .stdin(Stdio::null())
        .stdout(Stdio::from(io::stderr()))
        .stderr(Stdio::inherit())
        .status()
        .change_context(Failure("could not run cargo fmt"))?;
    if status.success() {
        Ok(())
    } else {
        Err(Report::new(Failure("cargo fmt failed")).attach(status.to_string()))
    }
}
pub fn run(invocation: &Invocation) -> Result<ExitCode, Report<Failure>> {
    let cargo = env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
    let mut child = Command::new(&cargo)
        .arg("clippy")
        .args(&invocation.arguments)
        .envs(&invocation.environment)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .change_context(Failure("could not start cargo clippy"))?;
    let Some(stdout) = child.stdout.take() else {
        return Err(Report::new(Failure("Cargo stdout pipe was not created")));
    };
    let mut reader = BufReader::new(stdout);
    let report_result = format(&mut reader, io::stdout().lock());
    let drain_result = io::copy(&mut reader, &mut io::sink())
        .change_context(Failure("could not drain Cargo output"));
    let status = child
        .wait()
        .change_context(Failure("could not wait for cargo clippy"))?;
    if let Err(error) = report_result {
        eprintln!("error: {error:?}");
    }
    if let Err(error) = drain_result {
        eprintln!("error: {error:?}");
    }
    if let Err(error) = run_formatter(&cargo, invocation) {
        eprintln!("error: {error:?}");
    }
    let code = status
        .code()
        .ok_or_else(|| Report::new(Failure("cargo clippy terminated without an exit code")))?;
    let byte = u8::try_from(code)
        .change_context(Failure("Cargo exit code cannot be represented"))
        .attach_with(|| format!("exit code: {code}"))?;
    Ok(ExitCode::from(byte))
}
