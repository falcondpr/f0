mod agents;
mod appearance;
mod commands;
mod config;
mod edit;
mod error;
mod git;
mod init;
mod launch;
mod layout;
mod process;
mod prompt;
mod tmux;
mod treehouse;

use std::path::PathBuf;

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
    /// Create the project file for a repo, asking about it.
    Init {
        /// A folder inside the repo; defaults to the current directory.
        path: Option<PathBuf>,
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
    /// Lease a treehouse slot on the task's branch, creating it unless it exists, and open
    /// its workspace.
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
        /// AI agent for this task only, like `codex`, overriding the project's. Repeat it
        /// to stack several agents in the session, the first on top.
        #[arg(long = "agent", value_name = "COMMAND")]
        agents: Vec<String>,
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
        /// AI agent for the rebuilt session, overriding the ones it had; repeatable, as in
        /// `f0 new`.
        #[arg(long = "agent", value_name = "COMMAND")]
        agents: Vec<String>,
        /// Create the session without attaching to it.
        #[arg(long)]
        no_attach: bool,
    },
    /// Start another agent in the session of a task, under the ones it has. Run it in one
    /// of the session's panes.
    Agent {
        /// The agent and its arguments, like `codex --full-auto`; defaults to the
        /// project's.
        #[arg(
            trailing_var_arg = true,
            allow_hyphen_values = true,
            value_name = "COMMAND"
        )]
        cmd: Vec<String>,
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
    /// The home screen of f0.app: pick a project, then a task to open or start. Leaving
    /// the task's session comes back to the tasks of its project.
    Launch,
    /// Pick an f0 session and switch to it.
    Switch,
    /// Close a slot's session and return the slot to the pool. Inside that session it asks
    /// in the status line and closes it in the background.
    Done {
        /// Slot id (`hubi/4`), task, session or path, as `f0 ls` shows them; defaults to the current directory.
        slot: Option<String>,
        /// Discard uncommitted changes without asking.
        #[arg(long)]
        force: bool,
    },
    /// Bind the key that closes a task on the running tmux server; `f0 new` and `f0 open`
    /// do it for you.
    Keys,
    /// Close a slot's session and keep the slot, with its branch and changes, for `f0 open`
    /// to bring back.
    Stop {
        /// Slot id (`hubi/4`), task, session or path, as `f0 ls` shows them; defaults to the current directory.
        slot: Option<String>,
    },
    /// The menu of the key that closes a task, run by tmux from the pane it was pressed in.
    #[command(hide = true)]
    CloseMenu { slot: PathBuf },
    /// Check tools, global defaults, project configs and the tmux keys.
    Doctor,
}

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Cmd::Init {
            path,
            name,
            template,
        } => {
            let dir = match path {
                Some(path) => path,
                None => std::env::current_dir()?,
            };
            init::run(&dir, name.as_deref(), template)
        }
        Cmd::Edit { project } => edit::run(project.as_deref()),
        Cmd::New {
            ticket,
            project,
            base,
            kind,
            start,
            agents,
            no_attach,
        } => commands::new(
            &ticket,
            project.as_deref(),
            base,
            kind.as_deref(),
            &start,
            &agents,
            attach_mode(no_attach),
        ),
        Cmd::Open {
            slot,
            start,
            agents,
            no_attach,
        } => commands::open(slot.as_deref(), &start, &agents, attach_mode(no_attach)),
        Cmd::Agent { cmd } => commands::agent(&cmd.join(" ")),
        Cmd::Ls { all, paths } => commands::ls(all, paths),
        Cmd::Launch => launch::run(),
        Cmd::Switch => commands::switch(),
        Cmd::Done { slot, force } => commands::done(slot.as_deref(), force),
        Cmd::Stop { slot } => commands::stop(slot.as_deref()),
        Cmd::CloseMenu { slot } => commands::close_menu(&slot),
        Cmd::Keys => commands::keys(),
        Cmd::Doctor => commands::doctor(),
    }
}

/// The CLI replaces itself with `tmux attach` unless told not to attach.
fn attach_mode(no_attach: bool) -> tmux::Attach {
    if no_attach {
        tmux::Attach::No
    } else {
        tmux::Attach::Replace
    }
}
