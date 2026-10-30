//! Pure description of a task window. `tmux::create_session` executes it.
//!
//! ```text
//! +------------------------------+-----------+
//! | editor                       | agent     |
//! +---------+---------+----------+-----------+
//! | term 0  | term 1  | term 2   | term 3    |
//! +---------+---------+----------+-----------+
//! ```

use std::path::{Path, PathBuf};

use crate::config::Project;

#[derive(Debug, Clone, PartialEq)]
pub struct Pane {
    pub title: String,
    pub cwd: PathBuf,
    pub command: Option<String>,
    /// Press Enter after typing `command`; otherwise it waits in the prompt.
    pub run: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// New pane to the right of the target.
    Right,
    /// New pane below the target.
    Below,
}

/// Splits `panes[target]`; the new pane is `panes[new]` and takes `percent` of the space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Split {
    pub target: usize,
    pub new: usize,
    pub direction: Direction,
    pub percent: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    /// `panes[0]` is the pane the session is created with.
    pub panes: Vec<Pane>,
    pub splits: Vec<Split>,
    pub focus: usize,
    /// Exported to every pane, so tools like nvim know the task context.
    pub env: Vec<(String, String)>,
}

const EDITOR: usize = 0;
const FIRST_TERMINAL: usize = 1;
const AGENT: usize = 2;

/// `start` names servers to run right away on top of the ones marked `autostart`.
/// `resume` starts the agent with its resume command, when it has one, falling back to
/// a fresh start.
pub fn build(project: &Project, worktree: &Path, start: &[String], resume: bool) -> Layout {
    let agent = match project.agent.resume_cmd().filter(|_| resume) {
        // Resume commands fail when there is nothing to resume, like `claude --continue`
        // in a worktree without conversations; the agent then starts fresh.
        Some(resume) => format!("{resume} || {}", project.agent.cmd),
        None => project.agent.cmd.clone(),
    };

    let terminals = usize::from(project.layout.terminals);

    let mut panes = vec![
        Pane {
            title: "editor".to_owned(),
            cwd: worktree.to_owned(),
            command: Some(project.editor.cmd.clone()),
            run: true,
        },
        terminal_pane(project, worktree, 0, start),
        Pane {
            title: format!("agent: {}", project.agent.program()),
            cwd: worktree.to_owned(),
            command: Some(agent),
            run: true,
        },
    ];

    // The terminal row is split off first so it spans the whole width, then the
    // agent column is carved out of the top row only.
    let mut splits = vec![
        Split {
            target: EDITOR,
            new: FIRST_TERMINAL,
            direction: Direction::Below,
            percent: project.layout.terminal_height,
        },
        Split {
            target: EDITOR,
            new: AGENT,
            direction: Direction::Right,
            percent: project.layout.agent_width,
        },
    ];

    let mut previous = FIRST_TERMINAL;
    for index in 1..terminals {
        let remaining = terminals - index;
        // Each split hands the new pane the share of every terminal still to come,
        // which leaves all terminals with the same width.
        let percent = (100 * remaining / (remaining + 1)) as u8;
        panes.push(terminal_pane(project, worktree, index, start));
        let new = panes.len() - 1;
        splits.push(Split {
            target: previous,
            new,
            direction: Direction::Right,
            percent,
        });
        previous = new;
    }

    let mut env = vec![("F0_PROJECT".to_owned(), project.name.clone())];
    if let Some(base) = &project.base {
        env.push(("F0_BASE".to_owned(), base.clone()));
    }

    Layout {
        panes,
        splits,
        focus: EDITOR,
        env,
    }
}

fn terminal_pane(project: &Project, worktree: &Path, index: usize, start: &[String]) -> Pane {
    match project.servers.get(index) {
        Some(server) => Pane {
            title: server.name.clone(),
            cwd: server
                .cwd
                .as_ref()
                .map_or_else(|| worktree.to_owned(), |cwd| worktree.join(cwd)),
            command: Some(server.cmd.clone()),
            run: server.autostart || start.contains(&server.name),
        },
        None => Pane {
            title: "shell".to_owned(),
            cwd: worktree.to_owned(),
            command: None,
            run: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Defaults;

    fn project(source: &str) -> Project {
        Project::from_toml(
            "hubi",
            source,
            Path::new("hubi.toml"),
            Path::new("/home/me"),
            &Defaults::default(),
        )
        .unwrap()
    }

    #[test]
    fn builds_default_layout_with_servers_then_shells() {
        let project = project(
            r#"
            repo = "/src/hubi"
            [[servers]]
            name = "nexus"
            cmd = "pnpm nx serve @org/nexus"
            [[servers]]
            name = "mobile"
            cwd = "apps/mobile"
            cmd = "npx expo start -c"
            "#,
        );
        let worktree = Path::new("/pool/1/hubi");
        let layout = build(&project, worktree, &[], false);

        let titles: Vec<_> = layout.panes.iter().map(|p| p.title.as_str()).collect();
        assert_eq!(
            titles,
            [
                "editor",
                "nexus",
                "agent: claude",
                "mobile",
                "shell",
                "shell"
            ]
        );
        assert_eq!(layout.panes[0].command.as_deref(), Some("nvim ."));
        assert_eq!(layout.panes[2].command.as_deref(), Some("claude"));
        assert_eq!(
            layout.panes[3].cwd,
            PathBuf::from("/pool/1/hubi/apps/mobile")
        );
        assert_eq!(layout.panes[4].command, None);
        assert!(layout.panes[0].run && layout.panes[2].run);
        assert!(!layout.panes[1].run && !layout.panes[3].run);
        assert_eq!(layout.focus, 0);
        assert_eq!(layout.env, [("F0_PROJECT".to_owned(), "hubi".to_owned())]);
    }

    #[test]
    fn start_and_autostart_pick_the_servers_that_run() {
        let project = project(
            r#"
            repo = "/src/hubi"
            [[servers]]
            name = "nexus"
            cmd = "a"
            autostart = true
            [[servers]]
            name = "admin"
            cmd = "b"
            [[servers]]
            name = "mobile"
            cmd = "c"
            "#,
        );
        let layout = build(&project, Path::new("/w"), &["mobile".to_owned()], false);
        let runs: Vec<_> = layout
            .panes
            .iter()
            .map(|p| (p.title.as_str(), p.run))
            .collect();
        assert_eq!(
            runs,
            [
                ("editor", true),
                ("nexus", true),
                ("agent: claude", true),
                ("admin", false),
                ("mobile", true),
                ("shell", false)
            ]
        );
    }

    #[test]
    fn resume_starts_the_agent_where_it_left_off() {
        let project = project(r#"repo = "/src/hubi""#);
        let agent = |resume| build(&project, Path::new("/w"), &[], resume).panes[AGENT].clone();
        assert_eq!(agent(false).command.as_deref(), Some("claude"));
        assert_eq!(
            agent(true).command.as_deref(),
            Some("claude --continue || claude")
        );
        assert_eq!(agent(true).title, "agent: claude");

        let codex = Project {
            agent: crate::config::Agent::new("codex"),
            ..project.clone()
        };
        let pane = build(&codex, Path::new("/w"), &[], true).panes[AGENT].clone();
        assert_eq!(pane.command.as_deref(), Some("codex"));
    }

    #[test]
    fn terminal_splits_share_the_row_evenly() {
        let project = project(r#"repo = "/src/hubi""#);
        let layout = build(&project, Path::new("/w"), &[], false);

        let row: Vec<_> = layout.splits[2..]
            .iter()
            .map(|s| (s.target, s.new, s.percent))
            .collect();
        assert_eq!(row, [(1, 3, 75), (3, 4, 66), (4, 5, 50)]);
        assert_eq!(layout.splits[0].direction, Direction::Below);
        assert_eq!(layout.splits[1].direction, Direction::Right);
    }

    #[test]
    fn single_terminal_has_no_row_splits() {
        let project = project("repo = \"/x\"\n[layout]\nterminals = 1");
        let layout = build(&project, Path::new("/w"), &[], false);
        assert_eq!(layout.panes.len(), 3);
        assert_eq!(layout.splits.len(), 2);
    }
}
