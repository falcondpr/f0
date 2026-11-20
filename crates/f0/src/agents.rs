//! The agents of a slot's session, remembered so that `f0 open`, from the CLI or the
//! launcher, rebuilds the session with the same agents. The file lives in the git
//! folder of the slot's worktree: git never lists it, and it leaves the repo alone.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::error::F0Error;
use crate::git;

const FILE: &str = "f0-agents";

/// The agent commands saved for the slot at `worktree`, top to bottom; empty when none
/// were saved.
pub fn load(worktree: &Path) -> Result<Vec<String>, F0Error> {
    let path = file(worktree)?;
    match fs::read_to_string(&path) {
        Ok(text) => Ok(parse(&text)),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(Vec::new()),
        Err(source) => Err(F0Error::AgentsFile { path, source }),
    }
}

pub fn save(worktree: &Path, cmds: &[String]) -> Result<(), F0Error> {
    let path = file(worktree)?;
    fs::write(&path, format(cmds)).map_err(|source| F0Error::AgentsFile { path, source })
}

/// Forgets the agents of a returned slot, so its next task starts with the project's.
pub fn clear(worktree: &Path) -> Result<(), F0Error> {
    // A slot treehouse removed has nothing left to forget.
    if !worktree.exists() {
        return Ok(());
    }
    let path = file(worktree)?;
    match fs::remove_file(&path) {
        Err(e) if e.kind() != ErrorKind::NotFound => Err(F0Error::AgentsFile { path, source: e }),
        _ => Ok(()),
    }
}

fn file(worktree: &Path) -> Result<PathBuf, F0Error> {
    Ok(git::git_dir(worktree)?.join(FILE))
}

/// One command per line; blank lines are skipped.
fn parse(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}

fn format(cmds: &[String]) -> String {
    cmds.iter().map(|cmd| format!("{}\n", cmd.trim())).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_round_trip_one_per_line() {
        let cmds = ["claude".to_owned(), "codex --full-auto".to_owned()];
        assert_eq!(format(&cmds), "claude\ncodex --full-auto\n");
        assert_eq!(parse(&format(&cmds)), cmds);
        assert_eq!(parse("\n  claude  \n\n"), ["claude"]);
        assert!(parse("").is_empty());
    }
}
