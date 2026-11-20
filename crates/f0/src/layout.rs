//! Pure description of a task window. `tmux::create_session` executes it.
//!
//! ```text
//! +------------------------------+-----------+
//! | editor                       | agent     |
//! |                              +-----------+
//! |                              | agent 2   |
//! +---------+---------+----------+-----------+
//! | term 0  | term 1  | term 2   | term 3    |
//! +---------+---------+----------+-----------+
//! ```

use std::path::{Path, PathBuf};

use crate::config::{Agent, Project};

#[derive(Debug, Clone, PartialEq)]
pub struct Pane {
    pub title: String,
    pub cwd: PathBuf,
    pub command: Option<String>,
    /// Press Enter after typing `command`; otherwise it waits in the prompt.
    pub run: bool,
    /// When the shell exits, for example with Ctrl-d at the prompt, a new one starts in
    /// the pane with this typed in its prompt, waiting for Enter, so the layout never
    /// loses the pane.
    pub restart: Option<String>,
    /// The command of the agent this pane runs, so f0 can find the agent panes of a
    /// running session and remember them.
    pub agent: Option<String>,
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
/// `agents` share the agent column, the first on top; it is never empty. `resume` starts
/// the first agent with its resume command, when it has one, falling back to a fresh
/// start. The others always start fresh: resuming by folder, like `claude --continue`,
/// cannot tell their conversations apart.
pub fn build(
    project: &Project,
    worktree: &Path,
    start: &[String],
    agents: &[Agent],
    resume: bool,
) -> Layout {
    let programs: Vec<&str> = agents.iter().map(Agent::program).collect();
    let mut titles = agent_titles(&programs).into_iter();
    let mut agent_panes = agents.iter().enumerate().map(|(index, agent)| {
        let title = titles.next().unwrap_or_default();
        agent_pane(agent, title, worktree, resume && index == 0)
    });
    let first_agent = agent_panes.next();

    let terminals = usize::from(project.layout.terminals);

    let mut panes = vec![
        Pane {
            title: "editor".to_owned(),
            cwd: worktree.to_owned(),
            command: Some(project.editor.cmd.clone()),
            run: true,
            restart: Some(project.editor.cmd.clone()),
            agent: None,
        },
        terminal_pane(project, worktree, 0, start),
    ];
    panes.extend(first_agent);

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
        panes.push(terminal_pane(project, worktree, index, start));
        let new = panes.len() - 1;
        splits.push(Split {
            target: previous,
            new,
            direction: Direction::Right,
            percent: even_share(terminals - index),
        });
        previous = new;
    }

    // The other agents go after the terminals, so the indexes above stay put, and stack
    // under the first one, sharing the height of the column.
    let mut previous = AGENT;
    let extra = agents.len().saturating_sub(1);
    for (index, pane) in agent_panes.enumerate() {
        panes.push(pane);
        let new = panes.len() - 1;
        splits.push(Split {
            target: previous,
            new,
            direction: Direction::Below,
            percent: even_share(extra - index),
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

/// The share of the target a split hands the new pane so that it and the `remaining - 1`
/// panes still to be split off it end up the same size.
fn even_share(remaining: usize) -> u8 {
    (100 * remaining / (remaining + 1)) as u8
}

/// The pane of an agent, in the worktree.
pub fn agent_pane(agent: &Agent, title: String, worktree: &Path, resume: bool) -> Pane {
    let start = agent.start_cmd();
    let command = match agent.resume_cmd().filter(|_| resume) {
        // Resume commands fail when there is nothing to resume, like `claude --continue`
        // in a worktree without conversations; the agent then starts fresh.
        Some(resume) => format!("{resume} || {start}"),
        None => start.clone(),
    };
    Pane {
        title,
        cwd: worktree.to_owned(),
        command: Some(command),
        run: true,
        // Leaving the agent and its shell is a request for a fresh one, not a resume.
        restart: Some(start),
        agent: Some(agent.cmd.clone()),
    }
}

/// Pane titles for agents running `programs`, top to bottom: `agent: claude`, then
/// `agent: claude 2` for a second one of the same program.
pub fn agent_titles(programs: &[&str]) -> Vec<String> {
    programs
        .iter()
        .enumerate()
        .map(
            |(index, program)| match programs[..index].iter().filter(|p| *p == program).count() {
                0 => format!("agent: {program}"),
                before => format!("agent: {program} {}", before + 1),
            },
        )
        .collect()
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
            restart: Some(server.cmd.clone()),
            agent: None,
        },
        None => Pane {
            title: "shell".to_owned(),
            cwd: worktree.to_owned(),
            command: None,
            run: false,
            restart: None,
            agent: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Defaults;

    const CLAUDE: &str = r#"claude --settings '{"theme":"auto"}'"#;

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
        let layout = build(
            &project,
            worktree,
            &[],
            std::slice::from_ref(&project.agent),
            false,
        );

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
        assert_eq!(layout.panes[2].command.as_deref(), Some(CLAUDE));
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
        let layout = build(
            &project,
            Path::new("/w"),
            &["mobile".to_owned()],
            std::slice::from_ref(&project.agent),
            false,
        );
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
        let agent = |resume| {
            build(
                &project,
                Path::new("/w"),
                &[],
                std::slice::from_ref(&project.agent),
                resume,
            )
            .panes[AGENT]
                .clone()
        };
        assert_eq!(agent(false).command.as_deref(), Some(CLAUDE));
        assert_eq!(
            agent(true).command,
            Some(format!("{CLAUDE} --continue || {CLAUDE}"))
        );
        assert_eq!(agent(true).title, "agent: claude");

        let codex = Project {
            agent: crate::config::Agent::new("codex"),
            ..project.clone()
        };
        let pane = build(
            &codex,
            Path::new("/w"),
            &[],
            std::slice::from_ref(&codex.agent),
            true,
        )
        .panes[AGENT]
            .clone();
        assert_eq!(pane.command.as_deref(), Some("codex"));
    }

    #[test]
    fn every_pane_survives_its_shell_with_its_command_ready() {
        let project = project(
            r#"
            repo = "/src/hubi"
            [[servers]]
            name = "nexus"
            cmd = "a"
            [[servers]]
            name = "mobile"
            cmd = "b"
            "#,
        );
        let restart: Vec<_> = build(
            &project,
            Path::new("/w"),
            &[],
            std::slice::from_ref(&project.agent),
            true,
        )
        .panes
        .into_iter()
        .map(|p| p.restart)
        .collect();
        assert_eq!(
            restart,
            [
                Some("nvim .".to_owned()),
                Some("a".to_owned()),
                Some(CLAUDE.to_owned()),
                Some("b".to_owned()),
                None,
                None
            ]
        );
    }

    #[test]
    fn terminal_splits_share_the_row_evenly() {
        let project = project(r#"repo = "/src/hubi""#);
        let layout = build(
            &project,
            Path::new("/w"),
            &[],
            std::slice::from_ref(&project.agent),
            false,
        );

        let row: Vec<_> = layout.splits[2..]
            .iter()
            .map(|s| (s.target, s.new, s.percent))
            .collect();
        assert_eq!(row, [(1, 3, 75), (3, 4, 66), (4, 5, 50)]);
        assert_eq!(layout.splits[0].direction, Direction::Below);
        assert_eq!(layout.splits[1].direction, Direction::Right);
    }

    fn agents(cmds: &[&str]) -> Vec<Agent> {
        cmds.iter().map(|cmd| Agent::new(*cmd)).collect()
    }

    #[test]
    fn one_agent_has_no_column_splits() {
        let project = project(r#"repo = "/src/hubi""#);
        let layout = build(&project, Path::new("/w"), &[], &agents(&["claude"]), false);
        assert_eq!(layout.panes.len(), 6);
        assert!(
            layout
                .splits
                .iter()
                .all(|split| split.direction == Direction::Right || split.target == EDITOR)
        );
        let marked: Vec<_> = layout.panes.iter().map(|p| p.agent.as_deref()).collect();
        assert_eq!(marked, [None, None, Some("claude"), None, None, None]);
    }

    #[test]
    fn two_agents_split_the_column_in_half() {
        let project = project("repo = \"/x\"\n[layout]\nterminals = 1");
        let layout = build(
            &project,
            Path::new("/w"),
            &[],
            &agents(&["claude", "codex"]),
            true,
        );
        let titles: Vec<_> = layout.panes.iter().map(|p| p.title.as_str()).collect();
        assert_eq!(titles, ["editor", "shell", "agent: claude", "agent: codex"]);
        assert_eq!(
            layout.splits[2],
            Split {
                target: AGENT,
                new: 3,
                direction: Direction::Below,
                percent: 50
            }
        );
        assert_eq!(layout.panes[3].cwd, PathBuf::from("/w"));
        assert_eq!(layout.panes[3].agent.as_deref(), Some("codex"));
    }

    #[test]
    fn three_agents_share_the_column_evenly_after_the_terminals() {
        let project = project(r#"repo = "/src/hubi""#);
        let layout = build(
            &project,
            Path::new("/w"),
            &[],
            &agents(&["claude", "codex", "claude"]),
            false,
        );
        let titles: Vec<_> = layout.panes[6..].iter().map(|p| p.title.as_str()).collect();
        assert_eq!(titles, ["agent: codex", "agent: claude 2"]);
        let column: Vec<_> = layout
            .splits
            .iter()
            .filter(|s| s.direction == Direction::Below && s.target != EDITOR)
            .map(|s| (s.target, s.new, s.percent))
            .collect();
        assert_eq!(column, [(AGENT, 6, 66), (6, 7, 50)]);
        let row: Vec<_> = layout.splits[2..5].iter().map(|s| s.new).collect();
        assert_eq!(row, [3, 4, 5], "terminals keep their indexes");
    }

    #[test]
    fn only_the_first_agent_resumes() {
        let project = project("repo = \"/x\"\n[layout]\nterminals = 1");
        let layout = build(
            &project,
            Path::new("/w"),
            &[],
            &agents(&["claude", "claude"]),
            true,
        );
        assert_eq!(
            layout.panes[AGENT].command,
            Some(format!("{CLAUDE} --continue || {CLAUDE}"))
        );
        assert_eq!(layout.panes[3].command.as_deref(), Some(CLAUDE));
        assert_eq!(layout.panes[3].restart.as_deref(), Some(CLAUDE));
        assert_eq!(layout.panes[3].title, "agent: claude 2");
    }

    #[test]
    fn agent_titles_number_repeated_programs() {
        assert_eq!(agent_titles(&["claude"]), ["agent: claude"]);
        assert_eq!(
            agent_titles(&["claude", "codex", "claude", "claude", "codex"]),
            [
                "agent: claude",
                "agent: codex",
                "agent: claude 2",
                "agent: claude 3",
                "agent: codex 2"
            ]
        );
        assert!(agent_titles(&[]).is_empty());
    }

    #[test]
    fn even_share_leaves_every_pane_the_same_size() {
        assert_eq!(even_share(1), 50);
        assert_eq!(even_share(2), 66);
        assert_eq!(even_share(3), 75);
    }

    #[test]
    fn single_terminal_has_no_row_splits() {
        let project = project("repo = \"/x\"\n[layout]\nterminals = 1");
        let layout = build(
            &project,
            Path::new("/w"),
            &[],
            std::slice::from_ref(&project.agent),
            false,
        );
        assert_eq!(layout.panes.len(), 3);
        assert_eq!(layout.splits.len(), 2);
    }
}
