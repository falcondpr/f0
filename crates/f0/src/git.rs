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

/// Files with uncommitted changes, untracked ones included.
pub fn changed_files(path: &Path) -> Result<usize, F0Error> {
    let status = process::output("git", ["status", "--porcelain"], Some(path))?;
    Ok(status.lines().count())
}

/// Whether `branch` exists here, or only on `origin`, where `git switch` picks it up.
pub fn branch_exists(repo: &Path, branch: &str) -> bool {
    ["refs/heads", "refs/remotes/origin"].iter().any(|refs| {
        process::output(
            "git",
            [
                "rev-parse",
                "--verify",
                "--quiet",
                &format!("{refs}/{branch}"),
            ],
            Some(repo),
        )
        .is_ok()
    })
}

/// The checkout of the repository that has `branch` checked out, if any.
pub fn checkout_of(repo: &Path, branch: &str) -> Result<Option<PathBuf>, F0Error> {
    let list = process::output("git", ["worktree", "list", "--porcelain"], Some(repo))?;
    Ok(parse_checkout_of(&list, branch))
}

fn parse_checkout_of(list: &str, branch: &str) -> Option<PathBuf> {
    let wanted = format!("branch refs/heads/{branch}");
    list.split("\n\n").find_map(|entry| {
        let mut lines = entry.lines();
        let path = lines.next()?.strip_prefix("worktree ")?;
        lines
            .any(|line| line == wanted)
            .then(|| PathBuf::from(path))
    })
}

pub fn switch(path: &Path, branch: &str) -> Result<(), F0Error> {
    process::output("git", ["switch", branch], Some(path)).map(drop)
}

/// Current branch, or `None` when HEAD is detached.
pub fn current_branch(path: &Path) -> Result<Option<String>, F0Error> {
    let branch = process::output("git", ["branch", "--show-current"], Some(path))?;
    Ok((!branch.is_empty()).then_some(branch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_checkout_of_a_branch() {
        let list = "worktree /src/app\nHEAD 1a\nbranch refs/heads/main\n\n\
                    worktree /pool/1/app\nHEAD 2b\ndetached\n\n\
                    worktree /pool/2/app\nHEAD 3c\nbranch refs/heads/feature/APP-1\n";
        assert_eq!(
            parse_checkout_of(list, "feature/APP-1"),
            Some(PathBuf::from("/pool/2/app"))
        );
        assert_eq!(
            parse_checkout_of(list, "main"),
            Some(PathBuf::from("/src/app"))
        );
        assert_eq!(parse_checkout_of(list, "feature/APP"), None);
    }
}
