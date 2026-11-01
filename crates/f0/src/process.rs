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
    Ok(raw_output(program, args, cwd)?.trim().to_owned())
}

/// Like `output`, but only drops the final newline, for values where spaces count.
pub fn line_output<I, S>(program: &str, args: I, cwd: Option<&Path>) -> Result<String, F0Error>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let out = raw_output(program, args, cwd)?;
    Ok(out.strip_suffix('\n').unwrap_or(&out).to_owned())
}

fn raw_output<I, S>(program: &str, args: I, cwd: Option<&Path>) -> Result<String, F0Error>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(program);
    command.args(args).stdin(Stdio::null());
    if let Some(dir) = cwd {
        command.current_dir(dir);
    }

    let out = command
        .output()
        .map_err(|source| spawn_error(program, source))?;
    if !out.status.success() {
        return Err(F0Error::CommandFailed {
            command: describe(&command),
            stderr: String::from_utf8_lossy(&out.stderr).trim().to_owned(),
        });
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Runs `program` attached to the current terminal, so it can prompt the user.
pub fn interactive<I, S>(program: &str, args: I, cwd: Option<&Path>) -> Result<(), F0Error>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(program);
    command.args(args);
    if let Some(dir) = cwd {
        command.current_dir(dir);
    }

    let status = command
        .status()
        .map_err(|source| spawn_error(program, source))?;
    if !status.success() {
        return Err(F0Error::CommandFailed {
            command: describe(&command),
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

fn spawn_error(program: &str, source: std::io::Error) -> F0Error {
    if source.kind() == ErrorKind::NotFound {
        F0Error::MissingProgram {
            program: program.to_owned(),
        }
    } else {
        F0Error::Spawn {
            program: program.to_owned(),
            source,
        }
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
