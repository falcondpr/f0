mod commands;
mod config;
mod edit;
mod error;
mod git;
mod init;
mod layout;
mod process;
mod prompt;
mod tmux;
mod treehouse;

use clap::{Parser, Subcommand};

/// A worktree-first terminal workspace for AI-assisted development.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// Create the project file for the repo in the current directory, asking about it.
    Init {
        /// Project name, used for the file and the sessions; defaults to the repo folder name.
        #[arg(long)]
        name: Option<String>,
        /// Write a file with <...> placeholders instead of asking.
        #[arg(long)]
        template: bool,
    },
    /// Update a project file question by question, showing the changes before saving.
    Edit {
        /// Project to edit; defaults to the project of the repo in the current directory.
        project: Option<String>,
    },
    /// Lease a treehouse slot with a new branch and open its workspace.
    New {
        /// Task id, appended to the project's branch prefix (e.g. APP-123).
        ticket: String,
        /// Project name; defaults to the project whose repo contains the current directory.
        #[arg(short, long)]
        project: Option<String>,
        /// Branch to cut from, overriding `base` in the project config.
        #[arg(long)]
        base: Option<String>,
        /// Branch type, one of the project's `branch_prefixes` without the `/` (e.g. `fix`);
        /// defaults to the first one.
        #[arg(short = 't', long = "type", value_name = "TYPE")]
        kind: Option<String>,
        /// Servers to start right away, comma separated; the others wait for Enter.
        #[arg(long, value_delimiter = ',', value_name = "SERVERS")]
        start: Vec<String>,
        /// AI agent for this task only, like `codex`, overriding the project's.
        #[arg(long, value_name = "COMMAND")]
        agent: Option<String>,
        /// Create the session without attaching to it.
        #[arg(long)]
        no_attach: bool,
    },
    /// Reopen the workspace of a treehouse slot, attaching to its session if it is running.
    Open {
        /// Slot id (`hubi/4`), task, session or path, as `f0 ls` shows them; defaults to the current directory.
        slot: Option<String>,
        /// Servers to start right away when the session is rebuilt, comma separated.
        #[arg(long, value_delimiter = ',', value_name = "SERVERS")]
        start: Vec<String>,
        /// AI agent for the rebuilt session, overriding the project's.
        #[arg(long, value_name = "COMMAND")]
        agent: Option<String>,
        /// Create the session without attaching to it.
        #[arg(long)]
        no_attach: bool,
    },
    /// List the slots in use, as `<project>/<slot>`, with their task and session.
    Ls {
        /// Also list free slots.
        #[arg(long)]
        all: bool,
        /// Show the folder of each slot.
        #[arg(long)]
        paths: bool,
    },
    /// Pick an f0 session and switch to it.
    Switch,
    /// Close a slot's session and return the slot to the pool.
    Done {
        /// Slot id (`hubi/4`), task, session or path, as `f0 ls` shows them; defaults to the current directory.
        slot: Option<String>,
    },
    /// Check tools, global defaults and project configs.
    Doctor,
}

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Cmd::Init { name, template } => init::run(name.as_deref(), template),
        Cmd::Edit { project } => edit::run(project.as_deref()),
        Cmd::New {
            ticket,
            project,
            base,
            kind,
            start,
            agent,
            no_attach,
        } => commands::new(
            &ticket,
            project.as_deref(),
            base,
            kind.as_deref(),
            &start,
            agent,
            !no_attach,
        ),
        Cmd::Open {
            slot,
            start,
            agent,
            no_attach,
        } => commands::open(slot.as_deref(), &start, agent, !no_attach),
        Cmd::Ls { all, paths } => commands::ls(all, paths),
        Cmd::Switch => commands::switch(),
        Cmd::Done { slot } => commands::done(slot.as_deref()),
        Cmd::Doctor => commands::doctor(),
    }
}
