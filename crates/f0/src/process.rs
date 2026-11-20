use std::ffi::OsStr;
use std::io::ErrorKind;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::error::F0Error;

/// Runs `program` and returns its trimmed stdout, failing on a non-zero exit.
pub fn output<I, S>(program: &str, args: I, cwd: Option<&Path>) -> Result<String, F0Error>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Ok(capture(&mut command(program, args, cwd))?.trim().to_owned())
}

/// Like `output`, but only drops the final newline, for values where spaces count.
pub fn line_output<I, S>(program: &str, args: I, cwd: Option<&Path>) -> Result<String, F0Error>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let out = capture(&mut command(program, args, cwd))?;
    Ok(out.strip_suffix('\n').unwrap_or(&out).to_owned())
}

/// The command `output` runs, for callers that need to adjust it first.
pub fn command<I, S>(program: &str, args: I, cwd: Option<&Path>) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(program);
    command.args(args);
    if let Some(dir) = cwd {
        command.current_dir(dir);
    }
    command
}

/// Runs `command` and returns its stdout, failing on a non-zero exit.
pub fn capture(command: &mut Command) -> Result<String, F0Error> {
    let out = command
        .stdin(Stdio::null())
        .output()
        .map_err(|source| spawn_error(command, source))?;
    if !out.status.success() {
        return Err(F0Error::CommandFailed {
            command: describe(command),
            stderr: String::from_utf8_lossy(&out.stderr).trim().to_owned(),
        });
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Runs `command` attached to the current terminal, so it can prompt the user.
pub fn attach(command: &mut Command) -> Result<(), F0Error> {
    let status = command
        .status()
        .map_err(|source| spawn_error(command, source))?;
    if !status.success() {
        return Err(F0Error::CommandFailed {
            command: describe(command),
            stderr: format!("exited with {status}"),
        });
    }
    Ok(())
}

pub fn is_installed(program: &str) -> bool {
    Command::new("which")
        .arg(program)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn spawn_error(command: &Command, source: std::io::Error) -> F0Error {
    let program = command.get_program().to_string_lossy().into_owned();
    if source.kind() == ErrorKind::NotFound {
        F0Error::MissingProgram { program }
    } else {
        F0Error::Spawn { program, source }
    }
}

fn describe(command: &Command) -> String {
    let args: Vec<_> = command.get_args().map(|a| a.to_string_lossy()).collect();
    format!(
        "{} {}",
        command.get_program().to_string_lossy(),
        args.join(" ")
    )
}
