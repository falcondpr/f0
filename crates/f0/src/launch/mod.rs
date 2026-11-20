//! `f0 launch`: the home screen of f0.app. Pick a project, then a task to open or start;
//! leaving its tmux session comes back here. `f0 switch` reuses the list.

mod scan;
mod state;
mod view;

use std::io::{self, BufRead, IsTerminal, Stdout, Write};
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::bail;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, execute};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::appearance::{self, Mode};
use crate::config::{self, Launch, Project};
use crate::tmux::{self, Attach, Session};
use crate::{commands, edit, init, treehouse};
use state::{Action, Key, ProjectItem, RepoItem, State, TaskItem};
use view::Palette;

type Term = Terminal<CrosstermBackend<Stdout>>;

/// Runs the launcher until `q`. It stays alive while you work in a session, so leaving
/// the session brings its list back.
pub fn run() -> anyhow::Result<()> {
    if std::env::var_os("TMUX").is_some() {
        bail!(
            "f0 launch is the home screen of a terminal outside tmux, and this shell runs \
             inside a tmux session. Use `f0 switch` to change sessions, or detach first \
             (prefix, then d)"
        );
    }
    require_terminal()?;
    ignore_interrupt();
    treehouse::update_in_background();
    let config_dir = config::config_dir()?;
    let defaults = config::load_defaults(&config_dir)?;
    let app = App {
        home: config::home_dir()?,
        launch: config::load_launch(&config_dir)?,
        projects: config::load_projects(&config_dir, &defaults)?,
        config_dir,
        refresh: None,
    };
    run_loop(State::new(Vec::new()), Some(app))
}

/// `f0 switch`: pick one of `sessions` and switch to it.
pub fn switch(sessions: Vec<Session>) -> anyhow::Result<()> {
    require_terminal()?;
    let mut items: Vec<TaskItem> = sessions
        .into_iter()
        .map(|session| TaskItem {
            task: session.name.clone(),
            branch: String::new(),
            path: session.path,
            session: Some(session.name),
            activity: session.activity,
        })
        .collect();
    state::sort_tasks(&mut items);
    run_loop(State::switcher(items), None)
}

/// What the launcher needs besides its `State`; `None` in `f0 switch`.
struct App {
    config_dir: PathBuf,
    home: PathBuf,
    launch: Launch,
    projects: Vec<Project>,
    /// The lists being reloaded in the background, if any.
    refresh: Option<Refresh>,
}

/// A background reload of the tasks of `project` and of the projects, which `treehouse
/// status` makes slow: up to a second or two per project. Each shows as soon as it is
/// done, so the tasks do not wait for every project.
struct Refresh {
    project: String,
    tasks: Option<JoinHandle<anyhow::Result<Vec<TaskItem>>>>,
    projects: Option<JoinHandle<Vec<ProjectItem>>>,
}

impl App {
    fn reload_projects(&mut self) -> anyhow::Result<()> {
        let defaults = config::load_defaults(&self.config_dir)?;
        self.projects = config::load_projects(&self.config_dir, &defaults)?;
        Ok(())
    }

    fn project(&self, name: &str) -> Option<&Project> {
        self.projects.iter().find(|p| p.name == name)
    }
}

fn run_loop(mut state: State, mut app: Option<App>) -> anyhow::Result<()> {
    install_panic_hook();
    let mode_file = appearance::mode_file(&config::config_dir()?);
    state.appearance = appearance::read_mode(&mode_file);
    let mut term = enter()?;
    paint_terminal(state.appearance)?;
    let mut palette = Palette::new(state.appearance.is_dark());
    if let Some(app) = &app {
        let items = busy(
            &mut term,
            &mut state,
            palette,
            "Loading projects…",
            || project_items(&app.projects, &app.home),
        )?;
        state.set_projects(items, None);
    }
    loop {
        if let Some(app) = app.as_mut() {
            apply_refresh(&mut state, app);
        }
        term.draw(|frame| view::draw(frame, &state, palette, now()))?;
        // While a reload runs, wake up now and then to show it; keys work meanwhile.
        let refreshing = app.as_ref().is_some_and(|app| app.refresh.is_some());
        if refreshing && !event::poll(REFRESH_POLL)? {
            continue;
        }
        let Some(key) = next_key()? else {
            continue;
        };
        match state.handle(key) {
            Action::None => {}
            Action::Quit => break,
            Action::Switch(session) => {
                leave()?;
                tmux::attach(&session, Attach::Replace)?;
                return Ok(());
            }
            action => {
                let Some(app) = app.as_mut() else {
                    continue;
                };
                perform(action, &mut state, app, &mut term, palette)?;
                // The appearance may have changed while a session had the terminal, or
                // in another f0 window.
                state.appearance = appearance::read_mode(&mode_file);
                paint_terminal(state.appearance)?;
                palette = Palette::new(state.appearance.is_dark());
            }
        }
    }
    leave()?;
    Ok(())
}

/// Carries out an action that needs f0, tmux or the disk, and shows what came of it.
fn perform(
    action: Action,
    state: &mut State,
    app: &mut App,
    term: &mut Term,
    palette: Palette,
) -> io::Result<()> {
    match action {
        Action::ShowTasks(name) => {
            let Some(project) = app.project(&name) else {
                state.error(format!("no project named `{name}`"));
                return Ok(());
            };
            match busy(term, state, palette, "Loading tasks…", || {
                tasks_of(project)
            })? {
                Ok(tasks) => state.show_tasks(name, tasks),
                Err(e) => state.error(format!("{e:#}")),
            }
        }
        Action::Open(slot) => {
            let target = slot.to_string_lossy().into_owned();
            let result = suspended(term, || {
                commands::open(Some(&target), &[], &[], Attach::Wait)
            })?;
            after_session(state, app, result);
        }
        Action::New { project, ticket } => {
            let result = suspended(term, || {
                commands::new(&ticket, Some(&project), None, None, &[], &[], Attach::Wait)
            })?;
            after_session(state, app, result);
        }
        Action::SearchRepos => {
            let folders: Vec<PathBuf> = app
                .launch
                .search_folders(&app.home)
                .into_iter()
                .filter(|folder| folder.is_dir())
                .collect();
            let labels = folders
                .iter()
                .map(|folder| config::with_tilde(folder, &app.home))
                .collect();
            state.show_search(labels);
            let known: Vec<PathBuf> = app
                .projects
                .iter()
                .map(|p| p.repo.canonicalize().unwrap_or_else(|_| p.repo.clone()))
                .collect();
            let found = busy(term, state, palette, "Looking for repos…", || {
                scan::find_repos(&folders, &known)
            })?;
            let repos = found
                .into_iter()
                .map(|path| RepoItem {
                    label: config::with_tilde(&path, &app.home),
                    path,
                })
                .collect();
            state.show_repos(repos);
        }
        Action::Init(folder) => {
            let folder = expand_tilde(&folder, &app.home);
            let before: Vec<String> = app.projects.iter().map(|p| p.name.clone()).collect();
            let result = suspended(term, || init::run(&folder, None, false))?;
            if let Err(e) = app.reload_projects() {
                state.error(format!("{e:#}"));
                return Ok(());
            }
            let added = app
                .projects
                .iter()
                .find(|p| !before.contains(&p.name))
                .map(|p| p.name.clone());
            let app = &*app;
            let items = busy(term, state, palette, "Loading projects…", || {
                project_items(&app.projects, &app.home)
            })?;
            state.set_projects(items, added.as_deref());
            match (result, added) {
                (Err(e), _) => state.error(format!("{e:#}")),
                (Ok(()), Some(name)) => {
                    state.show_projects(Some(&name));
                    state.notice(format!("Added {name}. Press Enter to see its tasks."));
                }
                (Ok(()), None) => state.show_projects(None),
            }
        }
        Action::Stop(slot) => {
            let target = slot.to_string_lossy().into_owned();
            let result = suspended(term, || commands::stop(Some(&target)))?;
            after_session(state, app, result);
        }
        Action::Done(slot) => {
            let target = slot.to_string_lossy().into_owned();
            // treehouse asks here before discarding uncommitted changes.
            let result = suspended(term, || commands::done(Some(&target), false))?;
            if result.is_ok() {
                state.remove_task(&slot);
            }
            after_session(state, app, result);
        }
        Action::Edit(name) => {
            let result = suspended(term, || edit::run(Some(&name)))?;
            if let Err(e) = result.and_then(|()| app.reload_projects()) {
                state.error(format!("{e:#}"));
                return Ok(());
            }
            let app = &*app;
            let items = busy(term, state, palette, "Loading projects…", || {
                project_items(&app.projects, &app.home)
            })?;
            state.set_projects(items, None);
        }
        Action::Doctor => {
            suspended(term, || {
                if let Err(e) = commands::doctor() {
                    eprintln!("{e:#}");
                }
                wait_for_enter("Press Enter to go back to f0.");
            })?;
        }
        Action::Appearance(mode) => {
            if let Err(e) = apply_appearance(mode, &app.config_dir) {
                state.error(format!("Could not change the theme: {e:#}"));
            }
        }
        Action::None | Action::Quit | Action::Switch(_) => {}
    }
    Ok(())
}

/// Saves `mode` where config/tmux/appearance.sh and Neovim read it, and has tmux repaint
/// its panes now; Neovim and the agent follow within a few seconds.
fn apply_appearance(mode: Mode, config_dir: &Path) -> anyhow::Result<()> {
    appearance::write_mode(&appearance::mode_file(config_dir), mode)?;
    let script = config::xdg_config_home()?.join("tmux/appearance.sh");
    if script.is_file() && tmux::server_running() {
        tmux::refresh_appearance(&script)?;
    }
    Ok(())
}

/// Gives the terminal around the launcher the colors of `mode`, so its margin, titlebar
/// and cursor match the panes tmux paints; `System` hands them back to the terminal's
/// own theme, which follows macOS. Inside tmux the sequences would recolor a pane
/// instead, so they are left out there.
fn paint_terminal(mode: Mode) -> io::Result<()> {
    if std::env::var_os("TMUX").is_some() {
        return Ok(());
    }
    let sequences = match mode {
        Mode::System => "\x1b]110\x1b\\\x1b]111\x1b\\\x1b]112\x1b\\".to_owned(),
        Mode::Light | Mode::Dark => {
            let (text, background) = if mode.is_dark() {
                ("d4/d4/d4", "1f/1f/1f")
            } else {
                ("1f/1f/1f", "ff/ff/ff")
            };
            format!(
                "\x1b]10;rgb:{text}\x1b\\\x1b]11;rgb:{background}\x1b\\\x1b]12;rgb:{text}\x1b\\"
            )
        }
    };
    let mut out = io::stdout();
    out.write_all(sequences.as_bytes())?;
    out.flush()
}

/// Keeps output on screen until Enter, before the launcher takes the terminal back.
fn wait_for_enter(prompt: &str) {
    print!("\n{prompt}");
    let _ = io::stdout().flush();
    let _ = io::stdin().lock().read_line(&mut String::new());
}

/// How often the launcher checks on a background reload while no key is pressed.
const REFRESH_POLL: Duration = Duration::from_millis(100);

/// Back from a task's session, or after closing or finishing one from the list: the
/// task list of its project again. Which tasks run comes from tmux right away, so the
/// next task opens without waiting; the slots, which only treehouse knows, and the
/// projects that Esc goes back to reload in the background.
fn after_session(state: &mut State, app: &mut App, result: anyhow::Result<()>) {
    let state::Screen::Tasks { project } = &state.screen else {
        return;
    };
    let project = project.clone();
    let sessions = tmux::sessions().unwrap_or_default();
    state.set_tasks(with_sessions(&state.tasks, &sessions));
    let running = state.tasks.iter().filter(|t| t.session.is_some()).count();
    let stopped = state.tasks.len() - running;
    if let Some(item) = state.projects.iter_mut().find(|p| p.name == project) {
        item.counts = Some((running, stopped));
    }
    if let Err(e) = result {
        state.error(format!("{e:#}"));
    }
    // A newer reload replaces an older one, whose thread finishes unheard.
    app.refresh = app.project(&project).cloned().map(|current| {
        let (projects, home) = (app.projects.clone(), app.home.clone());
        Refresh {
            project,
            tasks: Some(std::thread::spawn(move || tasks_of(&current))),
            projects: Some(std::thread::spawn(move || project_items(&projects, &home))),
        }
    });
}

/// Shows each half of the reload once it is done. The tasks only replace the list when
/// it is still that project's.
fn apply_refresh(state: &mut State, app: &mut App) {
    let Some(refresh) = app.refresh.as_mut() else {
        return;
    };
    if let Some(job) = refresh.tasks.take_if(|job| job.is_finished()) {
        let showing = matches!(
            &state.screen,
            state::Screen::Tasks { project } if *project == refresh.project
        );
        match joined(job) {
            Ok(tasks) if showing => state.set_tasks(tasks),
            Err(e) if showing => state.error(format!("{e:#}")),
            _ => {}
        }
    }
    if let Some(job) = refresh.projects.take_if(|job| job.is_finished()) {
        state.set_projects(joined(job), None);
    }
    if refresh.tasks.is_none() && refresh.projects.is_none() {
        app.refresh = None;
    }
}

/// The result of a finished thread, passing its panic on.
fn joined<T>(job: JoinHandle<T>) -> T {
    job.join()
        .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
}

/// `tasks` with what `sessions` say about them: which run, and when each was last used.
fn with_sessions(tasks: &[TaskItem], sessions: &[Session]) -> Vec<TaskItem> {
    let mut tasks: Vec<TaskItem> = tasks
        .iter()
        .map(|task| {
            let session = sessions.iter().find(|s| s.path == task.path);
            TaskItem {
                session: session.map(|s| s.name.clone()),
                activity: session.and_then(|s| s.activity),
                ..task.clone()
            }
        })
        .collect();
    state::sort_tasks(&mut tasks);
    tasks
}

/// Runs `job` on another thread while the spinner turns with `label`. Keys pressed
/// meanwhile are dropped.
fn busy<T: Send>(
    term: &mut Term,
    state: &mut State,
    palette: Palette,
    label: &str,
    job: impl FnOnce() -> T + Send,
) -> io::Result<T> {
    state.busy = Some(label.to_owned());
    let result = std::thread::scope(|scope| {
        let handle = scope.spawn(job);
        while !handle.is_finished() {
            term.draw(|frame| view::draw(frame, state, palette, now()))?;
            state.tick = state.tick.wrapping_add(1);
            if event::poll(Duration::from_millis(80))? {
                event::read()?;
            }
        }
        Ok(handle
            .join()
            .unwrap_or_else(|panic| std::panic::resume_unwind(panic)))
    });
    state.busy = None;
    result
}

fn tasks_of(project: &Project) -> anyhow::Result<Vec<TaskItem>> {
    let listing = commands::slots(std::slice::from_ref(project), &tmux::sessions()?, false);
    if let Some((_, error)) = listing.errors.into_iter().next() {
        bail!(error);
    }
    let mut tasks: Vec<TaskItem> = listing
        .rows
        .into_iter()
        .map(|row| TaskItem {
            task: row.task,
            branch: row.branch,
            path: row.path,
            activity: row.session.as_ref().and_then(|s| s.activity),
            session: row.session.map(|s| s.name),
        })
        .collect();
    state::sort_tasks(&mut tasks);
    Ok(tasks)
}

/// The projects with how many tasks each has running and stopped. A missing tmux server
/// only means nothing runs.
fn project_items(projects: &[Project], home: &Path) -> Vec<ProjectItem> {
    let sessions = tmux::sessions().unwrap_or_default();
    let listing = commands::slots(projects, &sessions, false);
    projects
        .iter()
        .map(|project| {
            let failed = listing.errors.iter().any(|(name, _)| *name == project.name);
            let rows = listing
                .rows
                .iter()
                .filter(|row| row.project == project.name);
            let running = rows.clone().filter(|row| row.session.is_some()).count();
            let stopped = rows.filter(|row| row.session.is_none()).count();
            ProjectItem {
                name: project.name.clone(),
                repo: config::with_tilde(&project.repo, home),
                counts: (!failed).then_some((running, stopped)),
            }
        })
        .collect()
}

fn expand_tilde(path: &Path, home: &Path) -> PathBuf {
    match path.strip_prefix("~") {
        Ok(rest) => home.join(rest),
        Err(_) => path.to_owned(),
    }
}

fn require_terminal() -> anyhow::Result<()> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        bail!("this command shows a full-screen list, so it needs a terminal");
    }
    Ok(())
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

fn next_key() -> io::Result<Option<Key>> {
    match event::read()? {
        Event::Key(key) if key.kind == KeyEventKind::Press => Ok(map_key(key)),
        // A resize, among others: drawing again is enough.
        _ => Ok(None),
    }
}

fn map_key(key: KeyEvent) -> Option<Key> {
    let control = key.modifiers.contains(KeyModifiers::CONTROL);
    match key.code {
        KeyCode::Char(_) if control || key.modifiers.contains(KeyModifiers::ALT) => None,
        KeyCode::Char(c) => Some(Key::Char(c)),
        KeyCode::Enter => Some(Key::Enter),
        KeyCode::Esc => Some(Key::Esc),
        KeyCode::Up => Some(Key::Up),
        KeyCode::Down => Some(Key::Down),
        KeyCode::Left => Some(Key::Left),
        KeyCode::Right => Some(Key::Right),
        KeyCode::Backspace => Some(Key::Backspace),
        _ => None,
    }
}

/// Takes over the terminal: raw keys, on the alternate screen, without a cursor.
fn enter() -> io::Result<Term> {
    terminal::enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let mut term = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    term.hide_cursor()?;
    term.clear()?;
    Ok(term)
}

/// Gives the terminal back as it was.
fn leave() -> io::Result<()> {
    terminal::disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, cursor::Show)
}

/// Runs `job` with the terminal given back, for tmux and the questions of `f0 init`, then
/// takes the terminal over again.
fn suspended<T>(term: &mut Term, job: impl FnOnce() -> T) -> io::Result<T> {
    leave()?;
    let result = job();
    *term = enter()?;
    Ok(result)
}

/// Ctrl-C must not close the launcher, even while it hands the terminal to the questions
/// of `f0 init` or `f0 edit`. A handler that does nothing, unlike ignoring the signal, is
/// reset when a child process starts, so tmux and the editor still get their Ctrl-C.
fn ignore_interrupt() {
    extern "C" fn nothing(_: libc::c_int) {}
    let handler: extern "C" fn(libc::c_int) = nothing;
    // SAFETY: the handler is async-signal-safe: it does nothing.
    unsafe { libc::signal(libc::SIGINT, handler as libc::sighandler_t) };
}

/// A panic would otherwise leave the shell in raw mode on the alternate screen.
fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = leave();
        previous(info);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sessions_mark_the_running_tasks_and_sort_them_first() {
        let task = |name: &str, session: Option<&str>| TaskItem {
            task: name.to_owned(),
            branch: String::new(),
            path: PathBuf::from(format!("/pool/{name}")),
            session: session.map(str::to_owned),
            activity: None,
        };
        let sessions = [Session {
            name: "web/B".to_owned(),
            path: PathBuf::from("/pool/B"),
            activity: Some(7),
        }];
        let tasks = with_sessions(&[task("A", Some("web/A")), task("B", None)], &sessions);
        let summary: Vec<_> = tasks
            .iter()
            .map(|t| (t.task.as_str(), t.session.as_deref(), t.activity))
            .collect();
        assert_eq!(summary, [("B", Some("web/B"), Some(7)), ("A", None, None)]);
    }

    #[test]
    fn typed_paths_expand_the_home_folder() {
        let home = Path::new("/home/me");
        assert_eq!(
            expand_tilde(Path::new("~/Desktop/app"), home),
            PathBuf::from("/home/me/Desktop/app")
        );
        assert_eq!(
            expand_tilde(Path::new("~"), home),
            PathBuf::from("/home/me")
        );
        assert_eq!(
            expand_tilde(Path::new("/srv/app"), home),
            PathBuf::from("/srv/app")
        );
        assert_eq!(
            expand_tilde(Path::new("~bob/x"), home),
            PathBuf::from("~bob/x")
        );
    }

    #[test]
    fn keys_map_to_launcher_keys() {
        let key = |code, modifiers| map_key(KeyEvent::new(code, modifiers));
        // Ctrl-C does nothing: only `q` closes the launcher.
        assert_eq!(key(KeyCode::Char('c'), KeyModifiers::CONTROL), None);
        assert_eq!(
            key(KeyCode::Char('j'), KeyModifiers::NONE),
            Some(Key::Char('j'))
        );
        assert_eq!(
            key(KeyCode::Char('J'), KeyModifiers::SHIFT),
            Some(Key::Char('J'))
        );
        assert_eq!(key(KeyCode::Char('x'), KeyModifiers::ALT), None);
        assert_eq!(key(KeyCode::Enter, KeyModifiers::NONE), Some(Key::Enter));
        assert_eq!(key(KeyCode::F(1), KeyModifiers::NONE), None);
    }
}
