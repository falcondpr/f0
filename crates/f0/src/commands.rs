use std::io;
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};

use crate::config::{self, Agent, Project};
use crate::error::F0Error;
use crate::tmux::{Attach, Session};
use crate::treehouse::Slot;
use crate::{appearance, git, layout, process, tmux, treehouse};

pub fn new(
    ticket: &str,
    project_name: Option<&str>,
    base: Option<String>,
    kind: Option<&str>,
    start: &[String],
    agent: Option<String>,
    attach: Attach,
) -> anyhow::Result<()> {
    require_treehouse()?;

    let config_dir = config::config_dir()?;
    let defaults = config::load_defaults(&config_dir)?;
    let projects = config::load_projects(&config_dir, &defaults)?;
    let mut project = resolve_project(&projects, project_name, &config_dir)?.clone();
    let base_given = base.is_some();
    if base.is_some() {
        project.base = base;
    }
    if let Some(cmd) = agent {
        project.agent = Agent::new(cmd);
    }
    project.ensure_filled()?;
    project.check_start(start)?;
    let branch = project.branch_for(ticket, kind)?;
    require_agent(&project)?;

    let include = project.worktree_include();
    if !include.exists() {
        return Err(F0Error::MissingWorktreeInclude {
            project: project.name,
            path: include,
        }
        .into());
    }

    let name = tmux::session_name(&project.name, ticket);
    if tmux::has_session(&name) {
        return Err(F0Error::SessionExists { name }.into());
    }

    let worktree = lease_for(&project, &branch, &format!("f0:{name}"), base_given)?;
    start_session(
        &project,
        &name,
        &canonical(&worktree)?,
        start,
        false,
        attach,
    )
}

/// Leases a slot for `branch`. A branch that already exists, from a task returned
/// before it was merged or pushed from elsewhere, is checked out instead of created.
fn lease_for(
    project: &Project,
    branch: &str,
    holder: &str,
    base_given: bool,
) -> anyhow::Result<PathBuf> {
    if !git::branch_exists(&project.repo, branch) {
        eprintln!("Leasing a {} slot for {branch}...", project.name);
        return Ok(treehouse::lease(project, Some(branch), holder)?);
    }
    if let Some(path) = git::checkout_of(&project.repo, branch)? {
        return Err(F0Error::BranchCheckedOut {
            branch: branch.to_owned(),
            path,
        }
        .into());
    }
    if base_given {
        eprintln!("{branch} already exists, so --base does nothing");
    }
    eprintln!(
        "Leasing a {} slot for the existing branch {branch}...",
        project.name
    );
    let worktree = treehouse::lease(project, None, holder)?;
    if let Err(e) = git::switch(&worktree, branch) {
        // The slot is fresh and detached, so returning it loses nothing.
        let _ = treehouse::return_slot(&worktree, true);
        return Err(e.into());
    }
    Ok(worktree)
}

/// Reopens the workspace of a leased slot, for example after tmux restarted. Attaches to
/// its session when that is still running.
pub fn open(
    target: Option<&str>,
    start: &[String],
    agent: Option<String>,
    attach: Attach,
) -> anyhow::Result<()> {
    require_treehouse()?;
    let config_dir = config::config_dir()?;
    let defaults = config::load_defaults(&config_dir)?;
    let projects = config::load_projects(&config_dir, &defaults)?;
    let root = resolve_slot(target, &projects)?;

    if let Some(session) = tmux::sessions_for(&root)?.into_iter().next() {
        if !start.is_empty() || agent.is_some() {
            eprintln!(
                "{} is already running, so --start and --agent do nothing; close it with \
                 `f0 done` or tmux to rebuild it",
                session.name
            );
        }
        // A session an older f0 built may predate the key.
        bind_keys_or_warn(&[&session.name]);
        return attach_if(&session.name, attach);
    }

    let mut project =
        config::find_by_repo(&projects, &git::main_repo_root(&root)?, &config_dir)?.clone();
    if let Some(cmd) = agent {
        project.agent = Agent::new(cmd);
    }
    project.ensure_filled()?;
    project.check_start(start)?;
    require_agent(&project)?;
    if !is_treehouse_slot(&project.repo, &root)? {
        return Err(F0Error::NotASlot { path: root }.into());
    }

    let task = match git::current_branch(&root)? {
        Some(branch) => project.task_from_branch(&branch).to_owned(),
        None => detached_task(&root),
    };
    let name = tmux::session_name(&project.name, &task);
    if tmux::has_session(&name) {
        return Err(F0Error::SessionExists { name }.into());
    }
    start_session(&project, &name, &root, start, true, attach)
}

/// A treehouse slot of a project, as `f0 ls` and the launcher show it.
#[derive(Debug, Clone, PartialEq)]
pub struct SlotRow {
    pub project: String,
    /// `<project>/<slot>`, unique across projects.
    pub id: String,
    /// The task, from the branch, or `(detached)`.
    pub task: String,
    /// Empty when the slot is detached.
    pub branch: String,
    pub status: String,
    pub path: PathBuf,
    /// The f0 session running in the slot, if any.
    pub session: Option<Session>,
}

/// The slots of some projects, with what `f0 ls` says about the rest.
#[derive(Debug, Default, PartialEq)]
pub struct Slots {
    pub rows: Vec<SlotRow>,
    /// Projects with free slots and how many, when the free ones are left out.
    pub free: Vec<(String, usize)>,
    /// The projects treehouse could not list, with why.
    pub errors: Vec<(String, String)>,
}

/// The slots in use of `projects`, or every slot with `all`, matched to `sessions`.
pub fn slots(projects: &[Project], sessions: &[Session], all: bool) -> Slots {
    slots_with(projects, sessions, all, treehouse::status)
}

/// `slots` with the call that lists the slots of a repo passed in, for tests.
fn slots_with(
    projects: &[Project],
    sessions: &[Session],
    all: bool,
    status: impl Fn(&Path) -> Result<Vec<Slot>, F0Error>,
) -> Slots {
    let mut out = Slots::default();
    for project in projects {
        let slots = match status(&project.repo) {
            Ok(slots) => slots,
            Err(e) => {
                out.errors.push((project.name.clone(), e.to_string()));
                continue;
            }
        };
        let available = slots.iter().filter(|s| s.status == "available").count();
        if available > 0 && !all {
            out.free.push((project.name.clone(), available));
        }
        for slot in slots {
            if slot.status == "available" && !all {
                continue;
            }
            let path = canonical(&slot.path).unwrap_or(slot.path);
            let session = sessions.iter().find(|s| s.path == path).cloned();
            let task = if slot.branch.is_empty() {
                "(detached)".to_owned()
            } else {
                project.task_from_branch(&slot.branch).to_owned()
            };
            out.rows.push(SlotRow {
                project: project.name.clone(),
                id: slot_id(&project.name, &slot.name),
                task,
                branch: slot.branch,
                status: slot.status,
                path,
                session,
            });
        }
    }
    out
}

/// Slots in use, one row each, named `<project>/<slot>`: treehouse numbers slots per
/// repo, so the number alone repeats across projects.
pub fn ls(all: bool, paths: bool) -> anyhow::Result<()> {
    require_treehouse()?;
    let config_dir = config::config_dir()?;
    let defaults = config::load_defaults(&config_dir)?;
    let projects = config::load_projects(&config_dir, &defaults)?;
    if projects.is_empty() {
        println!(
            "No projects in {}. Add one with `f0 init`.",
            config_dir.join("projects").display()
        );
        return Ok(());
    }

    let listing = slots(&projects, &tmux::sessions()?, all);
    for (project, error) in &listing.errors {
        eprintln!("{project}: {error}");
    }
    let rows: Vec<Vec<String>> = listing
        .rows
        .into_iter()
        .map(|slot| {
            let session = slot.session.map_or_else(|| "-".to_owned(), |s| s.name);
            let mut row = vec![slot.id, slot.task, slot.status, session];
            if paths {
                row.push(slot.path.display().to_string());
            }
            row
        })
        .collect();

    if rows.is_empty() {
        println!("No tasks. Start one with `f0 new <ticket>`.");
    } else {
        let mut headers = vec!["ID", "TASK", "STATUS", "SESSION"];
        if paths {
            headers.push("PATH");
        }
        print!("{}", format_table(&headers, &rows));
    }
    if !listing.free.is_empty() {
        let summary: Vec<_> = listing
            .free
            .iter()
            .map(|(project, count)| format!("{project} {count}"))
            .collect();
        println!(
            "Free slots: {} (`f0 ls --all` lists them)",
            summary.join(", ")
        );
    }
    Ok(())
}

/// The name `f0 ls` shows for a slot and `open`/`done` accept: unique across projects.
fn slot_id(project: &str, slot: &str) -> String {
    format!("{project}/{slot}")
}

/// Left-aligned columns as wide as their longest cell, two spaces apart.
fn format_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    let mut widths: Vec<usize> = headers.iter().map(|h| h.chars().count()).collect();
    for row in rows {
        for (width, cell) in widths.iter_mut().zip(row) {
            *width = (*width).max(cell.chars().count());
        }
    }
    let line = |cells: Vec<&str>| {
        let last = cells.len().saturating_sub(1);
        let mut text = String::new();
        for (index, (cell, width)) in cells.iter().zip(&widths).enumerate() {
            if index == last {
                text.push_str(cell);
            } else {
                text.push_str(&format!("{cell:<width$}  "));
            }
        }
        text.push('\n');
        text
    };
    let mut out = line(headers.to_vec());
    for row in rows {
        out.push_str(&line(row.iter().map(String::as_str).collect()));
    }
    out
}

pub fn switch() -> anyhow::Result<()> {
    let sessions = tmux::sessions()?;
    if sessions.is_empty() {
        println!("No f0 sessions. Start a task with `f0 new <ticket>`.");
        return Ok(());
    }
    crate::launch::switch(sessions)
}

/// The job that closes a task from inside its session: it takes the client that asked
/// (`#{1}`) out of tmux and runs `#{2}`, a `tmux::terminal_command`, in its terminal.
/// Closing the session from a tmux job would detach that client anyway, and whatever the
/// job printed, including why returning the slot failed, would never be seen.
const TERMINAL_JOB: &str = "tmux detach-client -t #{q:1} -E #{q:2}";
/// `f0 done --force` inside a session no client shows: nobody can see its output.
/// `#{1}` is f0 and `#{2}` the slot.
const DONE_JOB: &str = "#{q:1} done --force #{q:2} >/dev/null 2>&1";
/// The return choice of the key's menu: `#{1}` is f0, `#{2}` the slot and `#{3}` the
/// pane the key was pressed in. It asks first, in the status line, like `f0 done` inside
/// the session.
const RETURN_JOB: &str = "TMUX_PANE=#{q:3} #{q:1} done #{q:2} 2>&1";
const DONE_FAILED: &str = "The slot is still leased; f0 ls lists it. Press Enter to close.";
const STOP_FAILED: &str = "The session may still be running. Press Enter to close.";

/// Closes the session of a slot and returns the slot to the pool.
pub fn done(target: Option<&str>, force: bool) -> anyhow::Result<()> {
    require_treehouse()?;
    let config_dir = config::config_dir()?;
    let defaults = config::load_defaults(&config_dir)?;
    let projects = config::load_projects(&config_dir, &defaults)?;
    let root = resolve_slot(target, &projects)?;
    if !is_treehouse_slot(&git::main_repo_root(&root)?, &root)? {
        return Err(F0Error::NotASlot { path: root }.into());
    }

    let sessions = tmux::sessions_for(&root)?;
    let current = tmux::current_session();
    if let Some(session) = sessions
        .iter()
        .find(|s| current.as_deref() == Some(s.name.as_str()))
    {
        return done_from_inside(&session.name, &root, force);
    }

    for session in &sessions {
        eprintln!("Closing session {}", session.name);
        tmux::kill_session(&session.name)?;
    }
    treehouse::return_slot(&root, force)?;
    eprintln!("Returned {}", root.display());
    Ok(())
}

/// Closing the session would kill this process halfway, so the rest runs in the terminal
/// of the session's client once it leaves tmux. treehouse cannot ask there before
/// discarding changes, so f0 asks first and warns about the uncommitted ones.
fn done_from_inside(session: &str, root: &Path, force: bool) -> anyhow::Result<()> {
    let exe = std::env::current_exe().context("could not locate the f0 binary")?;
    let (exe, root_text) = (exe.to_string_lossy(), root.to_string_lossy());
    let client = match tmux::client_of(session) {
        Ok(client) => client,
        Err(F0Error::NoClient { .. }) if force => {
            tmux::run_in_background(DONE_JOB, &[&exe, &root_text])?;
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };
    let command = tmux::terminal_command(&[&exe, "done", "--force", &root_text], DONE_FAILED);
    let args = [client.as_str(), command.as_str()];
    if force {
        tmux::run_in_background(TERMINAL_JOB, &args)?;
        return Ok(());
    }
    let prompt = match git::changed_files(root)? {
        0 => format!("Close {session} and return its slot? (y/n)"),
        changes => {
            format!("Close {session}? {changes} uncommitted change(s) will be discarded (y/n)")
        }
    };
    tmux::confirm_then_run(session, &prompt, TERMINAL_JOB, &args)?;
    Ok(())
}

/// Binds the key that closes a task on the running tmux server and shows it in every f0
/// session. `f0 new` and `f0 open` do it for their sessions; this is for the sessions
/// that were already running when f0 was updated.
pub fn keys() -> anyhow::Result<()> {
    if !tmux::server_running() {
        println!(
            "No tmux server running; `f0 new` and `f0 open` bind the keys when they start one."
        );
        return Ok(());
    }
    let sessions = tmux::sessions()?;
    let names: Vec<&str> = sessions.iter().map(|s| s.name.as_str()).collect();
    match bind_keys(&names)? {
        Some(key) => println!(
            "{} {key} closes the task of an f0 session ({} running)",
            tmux::prefix()?,
            names.len()
        ),
        None => println!("The key that closes a task is turned off in config.toml"),
    }
    Ok(())
}

/// The configured key that closes a task, bound on the tmux server and shown in the
/// status line of `sessions`. `None` when `[keys] done` is `false`.
fn bind_keys(sessions: &[&str]) -> anyhow::Result<Option<String>> {
    let key = config::load_keys(&config::config_dir()?)?.done;
    let exe = std::env::current_exe().context("could not locate the f0 binary")?;
    tmux::bind_done(key.as_deref(), &exe.to_string_lossy())?;
    for session in sessions {
        tmux::show_done_hint(session, key.as_deref())?;
    }
    Ok(key)
}

/// The sessions work without the key, and `f0 done` typed in a pane closes the task too,
/// so a failure here only warns.
fn bind_keys_or_warn(sessions: &[&str]) -> Option<String> {
    bind_keys(sessions)
        .inspect_err(|e| eprintln!("Could not bind the key that closes a task: {e:#}"))
        .ok()
        .flatten()
}

/// Closes the session of a slot and keeps the slot leased, with its branch and
/// uncommitted changes, for `f0 open` to rebuild.
pub fn stop(target: Option<&str>) -> anyhow::Result<()> {
    require_treehouse()?;
    let config_dir = config::config_dir()?;
    let defaults = config::load_defaults(&config_dir)?;
    let projects = config::load_projects(&config_dir, &defaults)?;
    let root = resolve_slot(target, &projects)?;
    let repo = git::main_repo_root(&root)?;
    if !is_treehouse_slot(&repo, &root)? {
        return Err(F0Error::NotASlot { path: root }.into());
    }

    let mut sessions = tmux::sessions_for(&root)?;
    if sessions.is_empty() {
        println!("{} has no session running", root.display());
        return Ok(());
    }
    let task = config::project_for_repo(&projects, &repo)
        .zip(git::current_branch(&root)?)
        .map_or_else(
            || root.display().to_string(),
            |(project, branch)| project.task_from_branch(&branch).to_owned(),
        );
    eprintln!("The slot stays leased; `f0 open {task}` brings the session back");
    // Closing the session this runs in ends this process, so it goes last.
    let current = tmux::current_session();
    sessions.sort_by_key(|s| current.as_deref() == Some(s.name.as_str()));
    for session in &sessions {
        eprintln!("Closing session {}", session.name);
        tmux::kill_session(&session.name)?;
    }
    Ok(())
}

/// What the key that closes a task runs, from the pane it was pressed in: a menu to
/// close the session keeping the slot, or returning it.
pub fn close_menu(root: &Path) -> anyhow::Result<()> {
    let pane = std::env::var("TMUX_PANE").context("the key that closes a task sets TMUX_PANE")?;
    let session = tmux::current_session().context("no tmux session owns TMUX_PANE")?;
    let exe = std::env::current_exe().context("could not locate the f0 binary")?;
    let (exe, root) = (exe.to_string_lossy(), root.to_string_lossy());
    let client = tmux::client_of(&session)?;
    let stop = tmux::terminal_command(&[&exe, "stop", &root], STOP_FAILED);
    tmux::menu(
        &session,
        &format!("Close {session}"),
        &[
            tmux::MenuItem {
                label: "Close, keep the slot",
                key: "k",
                template: TERMINAL_JOB,
                args: &[&client, &stop],
            },
            tmux::MenuItem {
                label: "Close and return the slot",
                key: "r",
                template: RETURN_JOB,
                args: &[&exe, &root, &pane],
            },
        ],
    )?;
    Ok(())
}

pub fn doctor() -> anyhow::Result<()> {
    let mut problems = 0;

    println!("Tools");
    for program in ["tmux", "git", "treehouse"] {
        problems += report(program, process::is_installed(program), true);
    }
    for (program, purpose) in [
        ("gh", "pull requests"),
        ("rg", "search in the bundled nvim config"),
    ] {
        problems += report(
            &format!("{program}, {purpose}"),
            process::is_installed(program),
            false,
        );
    }
    let ghostty =
        Path::new("/Applications/Ghostty.app").exists() || process::is_installed("ghostty");
    problems += report("Ghostty", ghostty, false);

    let config_dir = config::config_dir()?;
    println!("\nf0.app");
    let app_id = process::output(
        "defaults",
        [
            "read",
            &format!("{APP}/Contents/Info"),
            "CFBundleIdentifier",
        ],
        None,
    )
    .ok();
    problems += report(
        &app_check_label(app_id.as_deref()),
        app_id.as_deref() == Some(APP_ID),
        false,
    );
    let app_config = config_dir.join("ghostty.conf");
    problems += report(
        &format!("its Ghostty config {}", app_config.display()),
        app_config.is_file(),
        false,
    );
    println!("\nDefaults ({})", config_dir.join("config.toml").display());
    let defaults = match config::load_defaults(&config_dir) {
        Ok(defaults) => defaults,
        Err(e) => {
            println!("  ✗ {e}");
            bail!("doctor found problems");
        }
    };
    problems += report_command("editor", &defaults.editor.cmd);
    problems += report_command("agent", &defaults.agent.cmd);

    println!("\ntmux keys");
    problems += report_done_key(config::load_keys(&config_dir)?.done.as_deref())?;

    problems += report_appearance()?;

    println!("\nProjects ({})", config_dir.join("projects").display());
    let projects = match config::load_projects(&config_dir, &defaults) {
        Ok(projects) => projects,
        Err(e) => {
            println!("  ✗ {e}");
            bail!("doctor found problems");
        }
    };
    if projects.is_empty() {
        problems += report(
            "no projects yet, add one (see the Configuration docs page)",
            false,
            true,
        );
    }
    for project in &projects {
        let repo_ok = git::main_repo_root(&project.repo).is_ok();
        problems += report(
            &format!("{}: repo {}", project.name, project.repo.display()),
            repo_ok,
            true,
        );

        let include = project.worktree_include();
        problems += report(
            &format!("{}: worktree include {}", project.name, include.display()),
            include.exists(),
            true,
        );

        if let Err(e) = project.ensure_filled() {
            problems += report(&e.to_string(), false, true);
        }

        problems += report_command(&format!("{}: editor", project.name), &project.editor.cmd);
        problems += report_command(&format!("{}: agent", project.name), &project.agent.cmd);
    }

    if problems > 0 {
        bail!("doctor found {problems} problem(s)");
    }
    println!("\nAll good.");
    Ok(())
}

fn start_session(
    project: &Project,
    name: &str,
    worktree: &Path,
    start: &[String],
    resume: bool,
    attach: Attach,
) -> anyhow::Result<()> {
    let layout = layout::build(project, worktree, start, resume);
    if let Err(e) = tmux::create_session(name, worktree, &layout) {
        // A half-built session would block the next attempt with SessionExists.
        if tmux::has_session(name) {
            let _ = tmux::kill_session(name);
        }
        return Err(e.into());
    }
    eprintln!("Session {name} ready at {}", worktree.display());
    if let Some(key) = bind_keys_or_warn(&[name]) {
        eprintln!("{} {key} in the session closes the task", tmux::prefix()?);
    }
    attach_if(name, attach)
}

fn attach_if(name: &str, attach: Attach) -> anyhow::Result<()> {
    if attach == Attach::No {
        println!("{name}");
    }
    tmux::attach(name, attach)?;
    Ok(())
}

/// A missing agent would leave its pane at a "command not found" prompt.
fn require_agent(project: &Project) -> Result<(), F0Error> {
    let program = project.agent.program();
    if process::is_installed(program) {
        Ok(())
    } else {
        Err(F0Error::MissingProgram {
            program: program.to_owned(),
        })
    }
}

fn require_treehouse() -> Result<(), F0Error> {
    if process::is_installed("treehouse") {
        Ok(())
    } else {
        Err(F0Error::TreehouseRequired)
    }
}

/// The checkout that contains `dir`: the slot itself when `dir` is inside one.
fn workspace_root(dir: &Path) -> anyhow::Result<PathBuf> {
    Ok(canonical(&git::worktree_root(&canonical(dir)?)?)?)
}

/// What `open` and `done` take: a folder inside a slot, or a slot as `f0 ls` shows it
/// (slot number, session name or task). A path wins, like in `treehouse return`.
fn resolve_slot(target: Option<&str>, projects: &[Project]) -> anyhow::Result<PathBuf> {
    let Some(target) = target else {
        return workspace_root(&std::env::current_dir()?);
    };
    if Path::new(target).exists() {
        return workspace_root(Path::new(target));
    }

    let sessions = tmux::sessions()?;
    // (id, project, path) of every slot `target` names.
    let mut matches: Vec<(String, &str, PathBuf)> = Vec::new();
    for project in projects {
        for slot in treehouse::status(&project.repo)? {
            let id = slot_id(&project.name, &slot.name);
            let task = (!slot.branch.is_empty()).then(|| project.task_from_branch(&slot.branch));
            let path = canonical(&slot.path).unwrap_or(slot.path);
            let session = sessions.iter().find(|s| s.path == path);
            let named = id == target
                || slot.name == target
                || task == Some(target)
                || session.is_some_and(|s| s.name == target);
            // Two projects on the same repo list the same slot twice.
            if named && !matches.iter().any(|(_, _, seen)| *seen == path) {
                matches.push((id, project.name.as_str(), path));
            }
        }
    }

    // Slot numbers repeat across projects, so the repo you stand in breaks the tie.
    if matches.len() > 1 {
        let here = std::env::current_dir()
            .ok()
            .and_then(|dir| git::main_repo_root(&dir).ok())
            .and_then(|repo| config::project_for_repo(projects, &repo));
        if let Some(here) = here {
            matches.retain(|(_, project, _)| *project == here.name);
        }
    }

    match matches.as_slice() {
        [] => Err(F0Error::NoSuchSlot {
            target: target.to_owned(),
        }
        .into()),
        [(_, _, path)] => Ok(path.clone()),
        _ => Err(F0Error::AmbiguousSlot {
            target: target.to_owned(),
            ids: matches
                .iter()
                .map(|(id, _, _)| id.as_str())
                .collect::<Vec<_>>()
                .join(", "),
        }
        .into()),
    }
}

fn is_treehouse_slot(repo: &Path, root: &Path) -> Result<bool, F0Error> {
    Ok(treehouse::status(repo)?
        .iter()
        .any(|slot| canonical(&slot.path).is_ok_and(|path| path == root)))
}

fn resolve_project<'a>(
    projects: &'a [Project],
    name: Option<&str>,
    config_dir: &Path,
) -> Result<&'a Project, F0Error> {
    match name {
        Some(name) => config::find_by_name(projects, name, config_dir),
        None => {
            let cwd = std::env::current_dir().map_err(|source| F0Error::ReadConfig {
                path: PathBuf::from("."),
                source,
            })?;
            config::find_by_repo(projects, &git::main_repo_root(&cwd)?, config_dir)
        }
    }
}

fn canonical(path: &Path) -> io::Result<PathBuf> {
    path.canonicalize()
}

/// Slots live at `{pool}/{slot}/{repo}`, so a detached slot is named after its slot directory.
fn detached_task(worktree: &Path) -> String {
    let slot = worktree
        .parent()
        .and_then(Path::file_name)
        .map_or_else(|| "detached".into(), |s| s.to_string_lossy());
    format!("slot-{slot}")
}

/// The app install.sh builds, and the bundle id that tells it from another app there.
const APP: &str = "/Applications/f0.app";
const APP_ID: &str = "com.falcondpr.f0";

/// What `f0 doctor` says about `/Applications/f0.app`, given its bundle id.
fn app_check_label(id: Option<&str>) -> String {
    match id {
        Some(APP_ID) => APP.to_owned(),
        Some(other) => format!("{APP} is another app ({other}); move it and run install.sh"),
        None => format!("{APP} missing, run install.sh to build it"),
    }
}

/// Prints a check line and returns 1 when a required check failed.
fn report(label: &str, ok: bool, required: bool) -> u32 {
    let mark = match (ok, required) {
        (true, _) => "✓",
        (false, true) => "✗",
        (false, false) => "-",
    };
    let note = if !ok && !required { " (optional)" } else { "" };
    println!("  {mark} {label}{note}");
    u32::from(!ok && required)
}

/// Checks that the running tmux server has the key that closes a task.
fn report_done_key(key: Option<&str>) -> anyhow::Result<u32> {
    let Some(key) = key else {
        println!("  - close a task: turned off in config.toml");
        return Ok(0);
    };
    if !tmux::server_running() {
        return Ok(report(
            &format!("close a task with `{key}`: no tmux server running to check"),
            false,
            false,
        ));
    }
    let label = format!("{} {key} closes a task", tmux::prefix()?);
    Ok(match tmux::prefix_binding(key)? {
        Some(binding) if tmux::is_f0_binding(&binding) => report(&label, true, true),
        Some(_) => report(
            &format!("{label}: tmux binds it to something else; set `[keys] done` in config.toml"),
            false,
            true,
        ),
        None => report(&format!("{label}: not bound, run `f0 keys`"), false, true),
    })
}

/// Checks what keeps the bundled theme from applying, only where the bundled configs
/// are linked: your own tmux or Ghostty config is yours to theme.
fn report_appearance() -> anyhow::Result<u32> {
    let home = config::home_dir()?;
    let xdg = config::xdg_config_home()?;
    let tmux = appearance::is_bundled(&xdg.join("tmux"), Path::new("config/tmux"));
    let ghostty = appearance::is_bundled(
        &xdg.join("ghostty/config"),
        Path::new("config/ghostty/config"),
    );
    if !tmux && !ghostty {
        return Ok(0);
    }
    println!("\nAppearance");
    let mut problems = 0;
    if tmux {
        problems += report_tmux_theme();
    }
    if ghostty {
        problems += report_ghostty_themes(&xdg);
        report_ghostty_colors(&home, &xdg);
    }
    Ok(problems)
}

fn report_tmux_theme() -> u32 {
    let (min_major, min_minor) = appearance::MIN_TMUX;
    let version = process::output("tmux", ["-V"], None).unwrap_or_default();
    match appearance::tmux_version(&version) {
        Some(found) if found >= appearance::MIN_TMUX => {
            report(&format!("`{version}` has the theme options"), true, true)
        }
        Some(_) => report(
            &format!(
                "`{version}` has no theme options, update to {min_major}.{min_minor} or later"
            ),
            false,
            true,
        ),
        None => report(
            &format!("tmux version unknown, the theme needs {min_major}.{min_minor} or later"),
            false,
            false,
        ),
    }
}

/// The bundled Ghostty config names the f0 themes, which install.sh links next to it;
/// a repo updated without running install.sh again lacks them, and Ghostty then falls
/// back to its default colors.
fn report_ghostty_themes(xdg: &Path) -> u32 {
    let missing = appearance::missing_ghostty_themes(&xdg.join("ghostty/themes"));
    if missing.is_empty() {
        return report("Ghostty has the f0 themes", true, true);
    }
    report(
        &format!(
            "Ghostty lacks the themes {}, run install.sh again",
            missing.join(", ")
        ),
        false,
        true,
    )
}

/// Colors set in another Ghostty config file win over the bundled theme around tmux.
/// Only a warning: the panes are painted by tmux and look right anyway.
fn report_ghostty_colors(home: &Path, xdg: &Path) {
    let mut found = false;
    for file in appearance::other_ghostty_files(home, xdg) {
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        for (line, key) in appearance::color_overrides(&text) {
            found = true;
            println!(
                "  - {}:{line} sets `{key}`, which overrides the Ghostty theme; remove that line",
                file.display()
            );
        }
    }
    if !found {
        report("Ghostty colors come from the bundled theme", true, false);
    }
}

/// Checks the program a pane will run; a missing one would leave the pane broken.
fn report_command(label: &str, command: &str) -> u32 {
    let program = command.split_whitespace().next().unwrap_or_default();
    report(
        &format!("{label} `{program}`"),
        process::is_installed(program),
        true,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_columns_fit_their_longest_cell() {
        let rows = vec![
            vec![
                "hubi-website/1".to_owned(),
                "(detached)".to_owned(),
                "in-use".to_owned(),
            ],
            vec![
                "hubi/4".to_owned(),
                "HB-123".to_owned(),
                "leased".to_owned(),
            ],
        ];
        assert_eq!(
            format_table(&["ID", "TASK", "STATUS"], &rows),
            "ID              TASK        STATUS\n\
             hubi-website/1  (detached)  in-use\n\
             hubi/4          HB-123      leased\n"
        );
    }

    #[test]
    fn slot_ids_are_unique_across_projects() {
        assert_ne!(slot_id("hubi", "1"), slot_id("hubi-website", "1"));
        assert_eq!(slot_id("hubi", "4"), "hubi/4");
    }

    fn project(name: &str, repo: &str) -> Project {
        Project::from_toml(
            name,
            &format!("repo = \"{repo}\""),
            Path::new("p.toml"),
            Path::new("/home/me"),
            &config::Defaults::default(),
        )
        .unwrap()
    }

    fn slot(name: &str, status: &str, branch: &str) -> Slot {
        Slot {
            name: name.to_owned(),
            path: PathBuf::from(format!("/pool/{name}/hubi")),
            status: status.to_owned(),
            branch: branch.to_owned(),
            lease_holder: String::new(),
        }
    }

    fn fake_status(repo: &Path) -> Result<Vec<Slot>, F0Error> {
        match repo.to_str() {
            Some("/src/hubi") => Ok(vec![
                slot("1", "leased", "feature/HB-1"),
                slot("2", "available", ""),
                slot("3", "leased", ""),
            ]),
            _ => Err(F0Error::NotARepo {
                path: repo.to_owned(),
            }),
        }
    }

    #[test]
    fn slots_match_sessions_and_count_the_free_ones() {
        let projects = [project("hubi", "/src/hubi"), project("gone", "/src/gone")];
        let session = Session {
            name: "hubi/HB-1".to_owned(),
            path: PathBuf::from("/pool/1/hubi"),
            activity: Some(10),
        };
        let listing = slots_with(
            &projects,
            std::slice::from_ref(&session),
            false,
            fake_status,
        );
        let ids: Vec<_> = listing.rows.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids, ["hubi/1", "hubi/3"]);
        assert_eq!(listing.rows[0].task, "HB-1");
        assert_eq!(listing.rows[0].branch, "feature/HB-1");
        assert_eq!(listing.rows[0].session, Some(session));
        assert_eq!(listing.rows[1].task, "(detached)");
        assert_eq!(listing.rows[1].session, None);
        assert_eq!(listing.free, [("hubi".to_owned(), 1)]);
        assert_eq!(listing.errors.len(), 1);
        assert_eq!(listing.errors[0].0, "gone");
    }

    #[test]
    fn all_slots_include_the_free_ones_without_a_summary() {
        let projects = [project("hubi", "/src/hubi")];
        let listing = slots_with(&projects, &[], true, fake_status);
        assert_eq!(listing.rows.len(), 3);
        assert!(listing.free.is_empty());
    }

    #[test]
    fn app_check_tells_missing_from_foreign() {
        assert_eq!(app_check_label(Some(APP_ID)), "/Applications/f0.app");
        assert!(app_check_label(None).contains("run install.sh to build it"));
        assert!(app_check_label(Some("org.wezfurlong.wezterm")).contains("another app"));
    }

    #[test]
    fn detached_task_uses_slot_directory() {
        assert_eq!(detached_task(Path::new("/pool/3/hubi")), "slot-3");
    }
}
