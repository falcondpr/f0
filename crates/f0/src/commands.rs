use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};

use crate::config::{self, Agent, Project};
use crate::error::F0Error;
use crate::{git, layout, process, tmux, treehouse};

pub fn new(
    ticket: &str,
    project_name: Option<&str>,
    base: Option<String>,
    kind: Option<&str>,
    start: &[String],
    agent: Option<String>,
    attach: bool,
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
    attach: bool,
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

    let sessions = tmux::sessions()?;
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut free: Vec<(&str, usize)> = Vec::new();
    for project in &projects {
        let slots = match treehouse::status(&project.repo) {
            Ok(slots) => slots,
            Err(e) => {
                eprintln!("{}: {e}", project.name);
                continue;
            }
        };
        let available = slots.iter().filter(|s| s.status == "available").count();
        if available > 0 {
            free.push((&project.name, available));
        }
        for slot in slots {
            if slot.status == "available" && !all {
                continue;
            }
            let path = canonical(&slot.path).unwrap_or(slot.path);
            let session = sessions
                .iter()
                .find(|s| s.path == path)
                .map_or_else(|| "-".to_owned(), |s| s.name.clone());
            let task = if slot.branch.is_empty() {
                "(detached)".to_owned()
            } else {
                project.task_from_branch(&slot.branch).to_owned()
            };
            let mut row = vec![
                slot_id(&project.name, &slot.name),
                task,
                slot.status,
                session,
            ];
            if paths {
                row.push(path.display().to_string());
            }
            rows.push(row);
        }
    }

    if rows.is_empty() {
        println!("No tasks. Start one with `f0 new <ticket>`.");
    } else {
        let mut headers = vec!["ID", "TASK", "STATUS", "SESSION"];
        if paths {
            headers.push("PATH");
        }
        print!("{}", format_table(&headers, &rows));
    }
    if !all && !free.is_empty() {
        let summary: Vec<_> = free
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

    for (index, session) in sessions.iter().enumerate() {
        println!(
            "{:>3}) {:<28} {}",
            index + 1,
            session.name,
            session.path.display()
        );
    }
    print!("Session: ");
    io::stdout().flush()?;

    let mut answer = String::new();
    io::stdin().lock().read_line(&mut answer)?;
    let choice: usize = answer.trim().parse().context("expected a session number")?;
    let Some(session) = choice.checked_sub(1).and_then(|i| sessions.get(i)) else {
        bail!("no session number {choice}");
    };
    tmux::attach(&session.name)?;
    Ok(())
}

/// Closes the session of a slot and returns the slot to the pool.
/// The job `f0 done` leaves to tmux when it runs inside the session it closes: `#{1}` is
/// f0 and `#{2}` the slot. Its output has nowhere to go once the session is gone, so a
/// failure is shown as a message.
const DONE_JOB: &str = "#{q:1} done --force #{q:2} >/dev/null 2>&1 || tmux display-message -d 0 \
     'f0 done could not return the slot; run f0 ls and f0 done <id> in a terminal to see why'";

/// The menu choices of the key that closes a task, as `run-shell` jobs: `#{1}` is f0,
/// `#{2}` the slot and `#{3}` the pane the key was pressed in. Returning asks first, in
/// the status line, like `f0 done` inside the session.
const STOP_JOB: &str = "#{q:1} stop #{q:2} >/dev/null 2>&1 || tmux display-message -d 0 \
     'f0 stop could not close the session; run f0 stop <id> in a terminal to see why'";
const RETURN_JOB: &str = "TMUX_PANE=#{q:3} #{q:1} done #{q:2} 2>&1";

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

/// Closing the session would kill this process halfway, so tmux runs the rest in the
/// background. That job has no terminal for treehouse to ask in, so f0 asks first and
/// warns about the uncommitted changes the job will discard.
fn done_from_inside(session: &str, root: &Path, force: bool) -> anyhow::Result<()> {
    let exe = std::env::current_exe().context("could not locate the f0 binary")?;
    let (exe, root_text) = (exe.to_string_lossy(), root.to_string_lossy());
    let args = [exe.as_ref(), root_text.as_ref()];
    if force {
        tmux::run_in_background(DONE_JOB, &args)?;
        return Ok(());
    }
    let prompt = match git::changed_files(root)? {
        0 => format!("Close {session} and return its slot? (y/n)"),
        changes => {
            format!("Close {session}? {changes} uncommitted change(s) will be discarded (y/n)")
        }
    };
    tmux::confirm_then_run(session, &prompt, DONE_JOB, &args)?;
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
    tmux::menu(
        &session,
        &format!("Close {session}"),
        &[
            tmux::MenuItem {
                label: "Close, keep the slot",
                key: "k",
                template: STOP_JOB,
                args: &[&exe, &root],
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
    attach: bool,
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

fn attach_if(name: &str, attach: bool) -> anyhow::Result<()> {
    if attach {
        tmux::attach(name)?;
    } else {
        println!("{name}");
    }
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

    #[test]
    fn detached_task_uses_slot_directory() {
        assert_eq!(detached_task(Path::new("/pool/3/hubi")), "slot-3");
    }
}
