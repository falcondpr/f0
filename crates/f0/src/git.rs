use std::path::{Path, PathBuf};

use crate::error::F0Error;
use crate::process;

/// Root of the checkout that contains `path` (the slot itself when inside a worktree).
pub fn worktree_root(path: &Path) -> Result<PathBuf, F0Error> {
    let root = process::output("git", ["rev-parse", "--show-toplevel"], Some(path))?;
    Ok(PathBuf::from(root))
}

/// Root of the main checkout, shared by every worktree of the repository.
pub fn main_repo_root(path: &Path) -> Result<PathBuf, F0Error> {
    let common = process::output(
        "git",
        ["rev-parse", "--path-format=absolute", "--git-common-dir"],
        Some(path),
    )?;
    let common = PathBuf::from(common);
    Ok(common.parent().map(Path::to_owned).unwrap_or(common))
}

/// The branch `origin/HEAD` points to, usually the repository's default branch.
pub fn default_branch(path: &Path) -> Option<String> {
    let head = process::output(
        "git",
        ["symbolic-ref", "--short", "refs/remotes/origin/HEAD"],
        Some(path),
    )
    .ok()?;
    Some(head.strip_prefix("origin/").unwrap_or(&head).to_owned())
}

/// The branch new tasks most likely start from: where `origin/HEAD` points, or the
/// current branch in a repo without a remote.
pub fn base_guess(path: &Path) -> Option<String> {
    default_branch(path).or_else(|| current_branch(path).ok().flatten())
}

/// Untracked, ignored `.env*` files of the checkout, the usual content of a
/// `.worktreeinclude`. Skips `node_modules`.
pub fn ignored_env_files(path: &Path) -> Result<Vec<String>, F0Error> {
    let files = process::output(
        "git",
        [
            "ls-files",
            "--others",
            "--ignored",
            "--exclude-standard",
            "--",
            ":(glob)**/.env*",
            ":(exclude,glob)**/node_modules/**",
        ],
        Some(path),
    )?;
    Ok(files.lines().map(str::to_owned).collect())
}

/// Current branch, or `None` when HEAD is detached.
pub fn current_branch(path: &Path) -> Result<Option<String>, F0Error> {
    let branch = process::output("git", ["branch", "--show-current"], Some(path))?;
    Ok((!branch.is_empty()).then_some(branch))
}
