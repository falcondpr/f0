use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde::Deserialize;

use crate::config::Project;
use crate::error::F0Error;
use crate::process;

const PROGRAM: &str = "treehouse";

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Slot {
    pub name: String,
    pub path: PathBuf,
    pub status: String,
    #[serde(default)]
    pub branch: String,
    #[serde(default)]
    pub lease_holder: String,
}

/// Leases a slot and returns its path: with a new branch cut from the project's base,
/// or detached at the base when `branch` is `None`.
pub fn lease(project: &Project, branch: Option<&str>, holder: &str) -> Result<PathBuf, F0Error> {
    let mut args: Vec<String> = vec![
        "get".into(),
        "--lease".into(),
        "--lease-holder".into(),
        holder.into(),
    ];
    if let Some(branch) = branch {
        args.extend(["--branch".into(), branch.into()]);
    }
    if let Some(base) = &project.base {
        args.extend(["--base".into(), base.clone()]);
    }
    if let Some(include) = &project.include_file {
        args.extend([
            "--include-file".into(),
            include.to_string_lossy().into_owned(),
        ]);
    }

    let path = capture(&args, Some(&project.repo))?;
    Ok(PathBuf::from(path.trim()))
}

pub fn status(repo: &Path) -> Result<Vec<Slot>, F0Error> {
    let json = capture(["status", "--json"], Some(repo))?;
    parse_status(&json)
}

/// Returns the slot to the pool. Runs attached to the terminal because treehouse
/// asks for confirmation before discarding uncommitted changes.
/// With `force` it discards them without asking.
pub fn return_slot(path: &Path, force: bool) -> Result<(), F0Error> {
    let mut args = vec!["return".as_ref(), path.as_os_str()];
    if force {
        args.push("--force".as_ref());
    }
    process::attach(&mut command(args, None))
}

/// Updates treehouse to its latest release without waiting for it: through Homebrew
/// when it installed treehouse, so its records stay right, or with `treehouse update`.
/// A failed update is ignored; the next launch tries again.
pub fn update_in_background() {
    let Ok(which) = process::output("which", [PROGRAM], None) else {
        return;
    };
    let Ok(binary) = std::fs::canonicalize(which) else {
        return;
    };
    let mut command = if binary.components().any(|c| c.as_os_str() == "Cellar") {
        let mut brew = Command::new("brew");
        brew.args(["upgrade", PROGRAM]);
        brew
    } else {
        let mut update = Command::new(&binary);
        update.arg("update");
        update
    };
    let spawned = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    if let Ok(mut child) = spawned {
        std::thread::spawn(move || child.wait());
    }
}

/// Runs treehouse and returns its stdout. A failure keeps only the error from stderr:
/// treehouse writes its progress there too, in lines that start with a tree.
fn capture<I, S>(args: I, cwd: Option<&Path>) -> Result<String, F0Error>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    process::capture(&mut command(args, cwd)).map_err(|err| match err {
        F0Error::CommandFailed { command, stderr } => F0Error::CommandFailed {
            command,
            stderr: without_progress(&stderr),
        },
        other => other,
    })
}

fn without_progress(stderr: &str) -> String {
    stderr
        .lines()
        .filter(|line| !line.trim_start().starts_with('🌳'))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_owned()
}

/// f0 keeps treehouse up to date itself, so the notice treehouse prints on stderr about
/// a newer release would only end up inside f0's errors.
fn command<I, S>(args: I, cwd: Option<&Path>) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = process::command(PROGRAM, args, cwd);
    command.env("TREEHOUSE_NO_UPDATE_CHECK", "1");
    command
}

fn parse_status(json: &str) -> Result<Vec<Slot>, F0Error> {
    serde_json::from_str(json).map_err(|source| F0Error::TreehouseStatus { source })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_status_json() {
        let json = r#"[
            {"name":"1","path":"/p/1/hubi","status":"leased","branch":"feature/HB-1","flavor":"git",
             "lease_id":"x","lease_holder":"f0:hubi/HB-1","leased_at":null,"processes":[]},
            {"name":"2","path":"/p/2/hubi","status":"available","detached":true,"flavor":"git"}
        ]"#;
        let slots = parse_status(json).unwrap();
        assert_eq!(slots.len(), 2);
        assert_eq!(slots[0].branch, "feature/HB-1");
        assert_eq!(slots[0].lease_holder, "f0:hubi/HB-1");
        assert_eq!(slots[1].branch, "");
        assert_eq!(slots[1].path, PathBuf::from("/p/2/hubi"));
    }

    #[test]
    fn drops_progress_from_errors() {
        let stderr =
            "🌳 Setting up worktree...\nall 16 worktrees are in use or dirty (max_trees = 16).";
        assert_eq!(
            without_progress(stderr),
            "all 16 worktrees are in use or dirty (max_trees = 16)."
        );
    }
}
