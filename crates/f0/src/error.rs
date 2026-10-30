use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum F0Error {
    #[error("`{program}` is not installed or not in PATH")]
    MissingProgram { program: String },

    #[error("could not run `{program}`: {source}")]
    Spawn {
        program: String,
        #[source]
        source: std::io::Error,
    },

    #[error("`{command}` failed: {stderr}")]
    CommandFailed { command: String, stderr: String },

    #[error("could not read {path}: {source}")]
    ReadConfig {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid config {path}: {source}")]
    ParseConfig {
        path: PathBuf,
        #[source]
        source: Box<toml::de::Error>,
    },

    #[error("project `{name}`: {reason}")]
    InvalidProject { name: String, reason: String },

    #[error("no project named `{name}` in {config_dir}")]
    UnknownProject { name: String, config_dir: PathBuf },

    #[error(
        "f0 does not know {repo} yet; describe it in {config_dir}/projects/<name>.toml with \
         `repo = \"{repo}\"` and run `f0 doctor` (see the Configuration docs page)"
    )]
    ProjectNotFound { repo: PathBuf, config_dir: PathBuf },

    #[error("could not parse `treehouse status --json`: {source}")]
    TreehouseStatus {
        #[source]
        source: serde_json::Error,
    },

    #[error("tmux session `{name}` already exists, use `f0 open` to attach")]
    SessionExists { name: String },

    #[error(
        "this pane belongs to the session being closed; run `f0 done {path}` from another terminal, \
         because returning the slot terminates every process inside it"
    )]
    InsideTargetSession { path: PathBuf },

    #[error(
        "f0 runs every task in its own treehouse worktree; install it with `brew install treehouse`"
    )]
    TreehouseRequired,

    #[error(
        "project `{project}` has no worktree include file at {path}; list the env files every \
         worktree needs there, or point `include_file` at one"
    )]
    MissingWorktreeInclude { project: String, path: PathBuf },

    #[error("{path} is not a treehouse slot; start a task with `f0 new <ticket>`")]
    NotASlot { path: PathBuf },

    #[error("`{target}` is not a folder, slot, session or task; run `f0 ls` to see them")]
    NoSuchSlot { target: String },

    #[error("`{target}` matches several slots; pass one of these instead: {ids}")]
    AmbiguousSlot { target: String, ids: String },

    #[error(
        "project `{project}` still has the placeholder `{value}` in `{key}`; edit its file, \
         then run `f0 doctor`"
    )]
    UnfilledPlaceholder {
        project: String,
        key: &'static str,
        value: String,
    },

    #[error("{path} is not inside a git repository; run `f0 init` from the repo you want to add")]
    NotARepo { path: PathBuf },

    #[error("project `{project}` has no server `{name}`; its servers are: {available}")]
    UnknownServer {
        project: String,
        name: String,
        available: String,
    },

    #[error("f0 edit asks questions, so it needs a terminal; edit {path} directly instead")]
    NeedsTerminal { path: PathBuf },

    #[error("project `{name}` already exists: {path}")]
    ProjectExists { name: String, path: PathBuf },

    #[error(
        "`{name}` cannot be a project name; pick one with `f0 init --name <name>` using \
         letters, digits, `-`, `_` or `.`"
    )]
    InvalidProjectName { name: String },

    #[error("could not determine the home directory")]
    NoHomeDir,
}
