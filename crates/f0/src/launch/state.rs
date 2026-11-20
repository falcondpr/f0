//! What the launcher shows and what each key does. Nothing here runs tmux, treehouse or
//! git: keys that need them return an `Action`, which `launch::run` carries out before
//! handing the results back through the `show_*` and `set_*` methods.

use std::path::PathBuf;

use crate::appearance::Mode;

/// The keys the launcher tells apart, independent of the terminal library.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Enter,
    Esc,
    Up,
    Down,
    Left,
    Right,
    Backspace,
}

/// A configured project.
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectItem {
    pub name: String,
    /// The repo folder, with `~` for the home folder.
    pub repo: String,
    /// Tasks with a tmux session, and tasks whose slot is leased without one. `None` when
    /// treehouse could not list the project's slots.
    pub counts: Option<(usize, usize)>,
}

/// A leased slot of the open project, or a session in `f0 switch`.
#[derive(Debug, Clone, PartialEq)]
pub struct TaskItem {
    pub task: String,
    /// Empty when the slot is detached or unknown.
    pub branch: String,
    pub path: PathBuf,
    /// The tmux session running in the slot; `None` when the task is stopped.
    pub session: Option<String>,
    /// When the session was last used, in seconds since the Unix epoch.
    pub activity: Option<u64>,
}

/// A repo found on disk that is not a project yet.
#[derive(Debug, Clone, PartialEq)]
pub struct RepoItem {
    pub path: PathBuf,
    /// The folder, with `~` for the home folder.
    pub label: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Projects,
    Tasks {
        project: String,
    },
    /// The repos found in `folders`, or a search in progress.
    AddRepo {
        folders: Vec<String>,
        searching: bool,
    },
    /// The running sessions, for `f0 switch`.
    Sessions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptKind {
    /// Narrows the current list as you type.
    Filter,
    /// The ticket of a new task.
    Ticket,
    /// The folder of a repo the search did not find.
    Path,
    /// A yes or no question about `State::pending`; the prompt text is the question.
    Confirm,
}

/// A line of text being typed at the bottom of the screen.
#[derive(Debug, Clone, PartialEq)]
pub struct Prompt {
    pub kind: PromptKind,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub text: String,
    pub error: bool,
}

/// What `launch::run` has to do after a key.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    None,
    Quit,
    /// Load the tasks of a project and show them.
    ShowTasks(String),
    /// Open the task in this slot and attach to it, like `f0 open`.
    Open(PathBuf),
    /// Start a task, like `f0 new`.
    New {
        project: String,
        ticket: String,
    },
    /// Look for repos to add.
    SearchRepos,
    /// Describe the repo in this folder, like `f0 init`. A typed path may start with `~`.
    Init(PathBuf),
    /// Switch to this session, for `f0 switch`.
    Switch(String),
    /// Edit a project file question by question, like `f0 edit`.
    Edit(String),
    /// Check the setup, like `f0 doctor`.
    Doctor,
    /// Save and apply the light or dark mode the workspace uses.
    Appearance(Mode),
    /// Close the session of the task in this slot and keep the slot, like `f0 stop`.
    Stop(PathBuf),
    /// Close the task in this slot and return the slot, like `f0 done`.
    Done(PathBuf),
}

#[derive(Debug, Clone, PartialEq)]
pub struct State {
    pub screen: Screen,
    pub projects: Vec<ProjectItem>,
    pub tasks: Vec<TaskItem>,
    pub repos: Vec<RepoItem>,
    /// Position of the selected row among the rows the filter keeps.
    pub selected: usize,
    pub filter: String,
    pub prompt: Option<Prompt>,
    /// What a `Confirm` prompt runs on yes.
    pub pending: Option<Action>,
    pub message: Option<Message>,
    /// How many cards fit in a row, which Up and Down jump over. Set from the window
    /// width before each key.
    pub columns: usize,
    /// What the launcher is waiting for, shown with a spinner.
    pub busy: Option<String>,
    /// Advances while busy, to animate the spinner.
    pub tick: usize,
    /// The light or dark mode, which `t` cycles on the projects screen.
    pub appearance: Mode,
}

impl State {
    /// The list of projects, the launcher's first screen.
    pub fn new(projects: Vec<ProjectItem>) -> Self {
        Self {
            screen: Screen::Projects,
            projects,
            tasks: Vec::new(),
            repos: Vec::new(),
            selected: 0,
            filter: String::new(),
            prompt: None,
            pending: None,
            message: None,
            columns: 1,
            busy: None,
            tick: 0,
            appearance: Mode::System,
        }
    }

    /// A list of running sessions to switch to, for `f0 switch`.
    pub fn switcher(sessions: Vec<TaskItem>) -> Self {
        Self {
            screen: Screen::Sessions,
            tasks: sessions,
            ..Self::new(Vec::new())
        }
    }

    pub fn show_tasks(&mut self, project: String, tasks: Vec<TaskItem>) {
        self.screen = Screen::Tasks { project };
        self.tasks = tasks;
        self.reset_list();
    }

    /// New tasks of the open project, keeping the selection where it was.
    pub fn set_tasks(&mut self, tasks: Vec<TaskItem>) {
        self.tasks = tasks;
        self.clamp();
    }

    /// New projects, selecting `select` when given.
    pub fn set_projects(&mut self, projects: Vec<ProjectItem>, select: Option<&str>) {
        self.projects = projects;
        if let Some(name) = select {
            self.filter.clear();
            self.selected = self
                .projects
                .iter()
                .position(|p| p.name == name)
                .unwrap_or(0);
        }
        self.clamp();
    }

    /// Back to the list of projects, with `select` selected when it is one.
    pub fn show_projects(&mut self, select: Option<&str>) {
        self.screen = Screen::Projects;
        self.reset_list();
        if let Some(name) = select {
            self.selected = self
                .projects
                .iter()
                .position(|p| p.name == name)
                .unwrap_or(0);
        }
    }

    pub fn show_search(&mut self, folders: Vec<String>) {
        self.screen = Screen::AddRepo {
            folders,
            searching: true,
        };
        self.repos.clear();
        self.reset_list();
    }

    pub fn show_repos(&mut self, repos: Vec<RepoItem>) {
        if let Screen::AddRepo { searching, .. } = &mut self.screen {
            *searching = false;
        }
        self.repos = repos;
        self.reset_list();
    }

    pub fn error(&mut self, text: impl Into<String>) {
        self.message = Some(Message {
            text: text.into(),
            error: true,
        });
    }

    pub fn notice(&mut self, text: impl Into<String>) {
        self.message = Some(Message {
            text: text.into(),
            error: false,
        });
    }

    /// Indexes, in the current list, of the rows the filter keeps.
    pub fn visible(&self) -> Vec<usize> {
        let needle = self.filter.to_lowercase();
        let keeps = |text: String| text.to_lowercase().contains(&needle);
        match self.screen {
            Screen::Projects => (0..self.projects.len())
                .filter(|&i| {
                    keeps(format!(
                        "{} {}",
                        self.projects[i].name, self.projects[i].repo
                    ))
                })
                .collect(),
            Screen::Tasks { .. } | Screen::Sessions => (0..self.tasks.len())
                .filter(|&i| keeps(format!("{} {}", self.tasks[i].task, self.tasks[i].branch)))
                .collect(),
            Screen::AddRepo { .. } => (0..self.repos.len())
                .filter(|&i| keeps(self.repos[i].label.clone()))
                .collect(),
        }
    }

    /// Index, in the current list, of the selected row.
    pub fn selected_index(&self) -> Option<usize> {
        self.visible().get(self.selected).copied()
    }

    pub fn handle(&mut self, key: Key) -> Action {
        self.message = None;
        if self.prompt.is_some() {
            return self.handle_prompt(key);
        }
        match key {
            Key::Up | Key::Char('k') => {
                if self.selected >= self.columns {
                    self.selected -= self.columns;
                }
            }
            Key::Down | Key::Char('j') => self.move_down(),
            Key::Left | Key::Char('h') => self.selected = self.selected.saturating_sub(1),
            Key::Right | Key::Char('l') => {
                let last = self.visible().len().saturating_sub(1);
                self.selected = (self.selected + 1).min(last);
            }
            Key::Char('/') => self.open_prompt(PromptKind::Filter, self.filter.clone()),
            Key::Char('q') => return Action::Quit,
            Key::Esc if !self.filter.is_empty() => {
                self.filter.clear();
                self.selected = 0;
            }
            Key::Esc => return self.back(),
            Key::Enter => return self.choose(),
            Key::Char('n') if matches!(self.screen, Screen::Tasks { .. }) => {
                self.open_prompt(PromptKind::Ticket, String::new());
            }
            Key::Char('a') if self.screen == Screen::Projects => return Action::SearchRepos,
            Key::Char('p') if matches!(self.screen, Screen::AddRepo { .. }) => {
                self.open_prompt(PromptKind::Path, String::new());
            }
            Key::Char('e') => return self.edit(),
            Key::Char('d') if self.screen == Screen::Projects => return Action::Doctor,
            Key::Char('t') if self.screen == Screen::Projects => {
                self.appearance = self.appearance.next();
                return Action::Appearance(self.appearance);
            }
            Key::Char('c') if matches!(self.screen, Screen::Tasks { .. }) => return self.stop(),
            Key::Char('x') if matches!(self.screen, Screen::Tasks { .. }) => self.ask_done(),
            _ => {}
        }
        Action::None
    }

    /// The selected project, or the open one on its tasks.
    fn edit(&self) -> Action {
        match &self.screen {
            Screen::Projects => self.selected_index().map_or(Action::None, |i| {
                Action::Edit(self.projects[i].name.clone())
            }),
            Screen::Tasks { project } => Action::Edit(project.clone()),
            Screen::AddRepo { .. } | Screen::Sessions => Action::None,
        }
    }

    fn stop(&mut self) -> Action {
        let Some(task) = self.selected_index().map(|i| &self.tasks[i]) else {
            return Action::None;
        };
        if task.session.is_none() {
            let text = format!("{} is already stopped", task.task);
            self.notice(text);
            return Action::None;
        }
        Action::Stop(task.path.clone())
    }

    fn ask_done(&mut self) {
        let Some(task) = self.selected_index().map(|i| &self.tasks[i]) else {
            return;
        };
        let question = format!("Finish {} and return its slot?", task.task);
        self.pending = Some(Action::Done(task.path.clone()));
        self.open_prompt(PromptKind::Confirm, question);
    }

    fn handle_prompt(&mut self, key: Key) -> Action {
        let Some(prompt) = &mut self.prompt else {
            return Action::None;
        };
        let kind = prompt.kind;
        if kind == PromptKind::Confirm {
            self.prompt = None;
            let pending = self.pending.take();
            return match key {
                Key::Char('y' | 'Y') => pending.unwrap_or(Action::None),
                _ => Action::None,
            };
        }
        match key {
            Key::Char(c) => prompt.text.push(c),
            Key::Backspace => {
                prompt.text.pop();
            }
            Key::Esc => {
                self.prompt = None;
                if kind == PromptKind::Filter {
                    self.filter.clear();
                    self.selected = 0;
                }
                return Action::None;
            }
            Key::Enter => return self.submit(),
            Key::Up | Key::Down | Key::Left | Key::Right => {}
        }
        if kind == PromptKind::Filter {
            self.filter = self
                .prompt
                .as_ref()
                .map(|p| p.text.clone())
                .unwrap_or_default();
            self.selected = 0;
        }
        Action::None
    }

    fn submit(&mut self) -> Action {
        let Some(prompt) = self.prompt.take() else {
            return Action::None;
        };
        let text = prompt.text.trim().to_owned();
        match prompt.kind {
            PromptKind::Filter | PromptKind::Confirm => Action::None,
            _ if text.is_empty() => {
                // Nothing typed yet: keep asking rather than close silently.
                self.prompt = Some(prompt);
                Action::None
            }
            PromptKind::Ticket => match &self.screen {
                Screen::Tasks { project } => Action::New {
                    project: project.clone(),
                    ticket: text,
                },
                _ => Action::None,
            },
            PromptKind::Path => Action::Init(PathBuf::from(text)),
        }
    }

    fn choose(&self) -> Action {
        let Some(index) = self.selected_index() else {
            return Action::None;
        };
        match &self.screen {
            Screen::Projects => Action::ShowTasks(self.projects[index].name.clone()),
            Screen::Tasks { .. } => Action::Open(self.tasks[index].path.clone()),
            Screen::AddRepo { .. } => Action::Init(self.repos[index].path.clone()),
            Screen::Sessions => match &self.tasks[index].session {
                Some(session) => Action::Switch(session.clone()),
                None => Action::None,
            },
        }
    }

    fn back(&mut self) -> Action {
        match &self.screen {
            Screen::Tasks { project } => {
                let project = project.clone();
                self.show_projects(Some(&project));
            }
            Screen::AddRepo { .. } => self.show_projects(None),
            Screen::Sessions => return Action::Quit,
            Screen::Projects => {}
        }
        Action::None
    }

    /// One row down, or to the last card when the row below is shorter.
    fn move_down(&mut self) {
        let Some(last) = self.visible().len().checked_sub(1) else {
            return;
        };
        let columns = self.columns.max(1);
        if self.selected + columns <= last {
            self.selected += columns;
        } else if self.selected / columns < last / columns {
            self.selected = last;
        }
    }

    pub fn set_columns(&mut self, columns: usize) {
        self.columns = columns.max(1);
    }

    fn open_prompt(&mut self, kind: PromptKind, text: String) {
        self.prompt = Some(Prompt { kind, text });
    }

    fn reset_list(&mut self) {
        self.selected = 0;
        self.filter.clear();
        self.prompt = None;
    }

    fn clamp(&mut self) {
        self.selected = self.selected.min(self.visible().len().saturating_sub(1));
    }
}

/// Running tasks first, most recently used on top, then the stopped ones by name.
pub fn sort_tasks(tasks: &mut [TaskItem]) {
    tasks.sort_by(|a, b| {
        b.session
            .is_some()
            .cmp(&a.session.is_some())
            .then(b.activity.cmp(&a.activity))
            .then_with(|| a.task.cmp(&b.task))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(name: &str) -> ProjectItem {
        ProjectItem {
            name: name.to_owned(),
            repo: format!("~/src/{name}"),
            counts: Some((0, 0)),
        }
    }

    fn task(name: &str, session: Option<&str>, activity: Option<u64>) -> TaskItem {
        TaskItem {
            task: name.to_owned(),
            branch: format!("feature/{name}"),
            path: PathBuf::from(format!("/pool/{name}")),
            session: session.map(str::to_owned),
            activity,
        }
    }

    fn typed(state: &mut State, text: &str) {
        for c in text.chars() {
            state.handle(Key::Char(c));
        }
    }

    fn tasks_screen() -> State {
        let mut state = State::new(vec![project("api"), project("web")]);
        state.show_tasks(
            "web".to_owned(),
            vec![
                task("WEB-1", Some("web/WEB-1"), Some(5)),
                task("WEB-2", None, None),
            ],
        );
        state
    }

    #[test]
    fn arrows_and_jk_move_within_the_list() {
        let mut state = State::new(vec![project("api"), project("web"), project("cli")]);
        state.handle(Key::Up);
        assert_eq!(state.selected, 0);
        state.handle(Key::Char('j'));
        state.handle(Key::Down);
        state.handle(Key::Down);
        assert_eq!(state.selected, 2);
        state.handle(Key::Char('k'));
        assert_eq!(state.selected, 1);
    }

    #[test]
    fn cards_move_in_a_grid() {
        let projects = (0..7).map(|i| project(&format!("p{i}"))).collect();
        let mut state = State::new(projects);
        state.set_columns(3);
        // p0 p1 p2
        // p3 p4 p5
        // p6
        state.handle(Key::Right);
        state.handle(Key::Char('l'));
        state.handle(Key::Right);
        assert_eq!(state.selected, 3, "right walks past the end of a row");
        state.handle(Key::Char('h'));
        assert_eq!(state.selected, 2);
        state.handle(Key::Down);
        assert_eq!(state.selected, 5);
        state.handle(Key::Down);
        assert_eq!(state.selected, 6, "a shorter last row takes its last card");
        state.handle(Key::Down);
        assert_eq!(state.selected, 6);
        state.handle(Key::Up);
        assert_eq!(state.selected, 3);
        state.handle(Key::Up);
        state.handle(Key::Up);
        assert_eq!(state.selected, 0, "the first row stays put");
        state.handle(Key::Left);
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn enter_on_a_project_asks_for_its_tasks() {
        let mut state = State::new(vec![project("api"), project("web")]);
        state.handle(Key::Down);
        assert_eq!(
            state.handle(Key::Enter),
            Action::ShowTasks("web".to_owned())
        );
    }

    #[test]
    fn enter_on_a_task_opens_its_slot() {
        let mut state = tasks_screen();
        state.handle(Key::Down);
        assert_eq!(
            state.handle(Key::Enter),
            Action::Open(PathBuf::from("/pool/WEB-2"))
        );
    }

    #[test]
    fn esc_goes_back_to_the_project_it_came_from() {
        let mut state = tasks_screen();
        assert_eq!(state.handle(Key::Esc), Action::None);
        assert_eq!(state.screen, Screen::Projects);
        assert_eq!(state.selected, 1);
        assert_eq!(state.handle(Key::Esc), Action::None);
        assert_eq!(state.screen, Screen::Projects);
    }

    #[test]
    fn q_quits_except_while_typing() {
        let mut state = tasks_screen();
        state.handle(Key::Char('n'));
        assert_eq!(state.handle(Key::Char('q')), Action::None);
        assert_eq!(state.prompt.as_ref().unwrap().text, "q");
        state.handle(Key::Esc);
        assert_eq!(state.handle(Key::Char('q')), Action::Quit);
    }

    #[test]
    fn n_asks_for_a_ticket_and_starts_the_task() {
        let mut state = tasks_screen();
        state.handle(Key::Char('n'));
        assert_eq!(state.handle(Key::Enter), Action::None, "empty ticket");
        assert!(state.prompt.is_some());
        typed(&mut state, "WEB-99x");
        state.handle(Key::Backspace);
        assert_eq!(
            state.handle(Key::Enter),
            Action::New {
                project: "web".to_owned(),
                ticket: "WEB-99".to_owned()
            }
        );
        assert!(state.prompt.is_none());
    }

    #[test]
    fn n_does_nothing_outside_the_tasks() {
        let mut state = State::new(vec![project("api")]);
        state.handle(Key::Char('n'));
        assert!(state.prompt.is_none());
    }

    #[test]
    fn filter_narrows_as_you_type_and_esc_clears_it() {
        let mut state = State::new(vec![project("api"), project("web"), project("webview")]);
        state.handle(Key::Char('/'));
        typed(&mut state, "WEB");
        assert_eq!(state.visible(), [1, 2]);
        state.handle(Key::Enter);
        assert!(state.prompt.is_none());
        assert_eq!(state.filter, "WEB");
        state.handle(Key::Down);
        assert_eq!(
            state.handle(Key::Enter),
            Action::ShowTasks("webview".to_owned())
        );

        state.handle(Key::Esc);
        assert_eq!(state.filter, "");
        assert_eq!(state.visible().len(), 3);
    }

    #[test]
    fn esc_while_filtering_drops_the_filter() {
        let mut state = State::new(vec![project("api"), project("web")]);
        state.handle(Key::Char('/'));
        typed(&mut state, "zzz");
        assert!(state.visible().is_empty());
        assert_eq!(state.handle(Key::Enter), Action::None);
        state.handle(Key::Char('/'));
        state.handle(Key::Esc);
        assert_eq!(state.visible().len(), 2);
    }

    #[test]
    fn enter_on_an_empty_list_does_nothing() {
        assert_eq!(State::new(Vec::new()).handle(Key::Enter), Action::None);
        let mut state = State::new(vec![project("api")]);
        state.show_tasks("api".to_owned(), Vec::new());
        assert_eq!(state.handle(Key::Enter), Action::None);
    }

    #[test]
    fn a_searches_for_repos_and_enter_adds_one() {
        let mut state = State::new(Vec::new());
        assert_eq!(state.handle(Key::Char('a')), Action::SearchRepos);
        state.show_search(vec!["~/Desktop".to_owned()]);
        assert_eq!(state.handle(Key::Enter), Action::None, "still searching");
        state.show_repos(vec![RepoItem {
            path: PathBuf::from("/home/me/Desktop/app"),
            label: "~/Desktop/app".to_owned(),
        }]);
        assert_eq!(
            state.screen,
            Screen::AddRepo {
                folders: vec!["~/Desktop".to_owned()],
                searching: false
            }
        );
        assert_eq!(
            state.handle(Key::Enter),
            Action::Init(PathBuf::from("/home/me/Desktop/app"))
        );
        state.handle(Key::Esc);
        assert_eq!(state.screen, Screen::Projects);
    }

    #[test]
    fn p_takes_a_typed_path() {
        let mut state = State::new(Vec::new());
        state.show_search(Vec::new());
        state.show_repos(Vec::new());
        state.handle(Key::Char('p'));
        typed(&mut state, " ~/Desktop/app ");
        assert_eq!(
            state.handle(Key::Enter),
            Action::Init(PathBuf::from("~/Desktop/app"))
        );
    }

    #[test]
    fn switcher_switches_and_esc_quits() {
        let mut state = State::switcher(vec![task("web/WEB-1", Some("web/WEB-1"), Some(1))]);
        assert_eq!(
            state.handle(Key::Enter),
            Action::Switch("web/WEB-1".to_owned())
        );
        assert_eq!(state.handle(Key::Esc), Action::Quit);
    }

    #[test]
    fn reloading_tasks_keeps_the_selection_in_range() {
        let mut state = tasks_screen();
        state.handle(Key::Down);
        state.set_tasks(vec![task("WEB-1", None, None)]);
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn new_projects_can_select_the_one_just_added() {
        let mut state = State::new(vec![project("api")]);
        state.set_projects(vec![project("api"), project("web")], Some("web"));
        assert_eq!(state.selected, 1);
    }

    #[test]
    fn running_tasks_come_first_by_last_use() {
        let mut tasks = vec![
            task("B", None, None),
            task("old", Some("s1"), Some(10)),
            task("A", None, None),
            task("new", Some("s2"), Some(20)),
        ];
        sort_tasks(&mut tasks);
        let names: Vec<_> = tasks.iter().map(|t| t.task.as_str()).collect();
        assert_eq!(names, ["new", "old", "A", "B"]);
    }

    #[test]
    fn task_keys_stop_finish_and_edit() {
        let mut state = tasks_screen();
        assert_eq!(
            state.handle(Key::Char('c')),
            Action::Stop(PathBuf::from("/pool/WEB-1"))
        );
        assert_eq!(state.handle(Key::Char('e')), Action::Edit("web".to_owned()));
        state.handle(Key::Down);
        assert_eq!(
            state.handle(Key::Char('c')),
            Action::None,
            "already stopped"
        );
        assert!(
            state
                .message
                .as_ref()
                .unwrap()
                .text
                .contains("already stopped")
        );
    }

    #[test]
    fn x_finishes_a_task_only_after_yes() {
        let mut state = tasks_screen();
        assert_eq!(state.handle(Key::Char('x')), Action::None);
        assert_eq!(state.prompt.as_ref().unwrap().kind, PromptKind::Confirm);
        assert!(state.prompt.as_ref().unwrap().text.contains("Finish WEB-1"));
        assert_eq!(state.handle(Key::Char('n')), Action::None);
        assert!(state.prompt.is_none() && state.pending.is_none());

        state.handle(Key::Char('x'));
        assert_eq!(
            state.handle(Key::Char('Y')),
            Action::Done(PathBuf::from("/pool/WEB-1"))
        );
        state.handle(Key::Char('x'));
        assert_eq!(state.handle(Key::Esc), Action::None);
        assert_eq!(
            state.screen,
            Screen::Tasks {
                project: "web".to_owned()
            }
        );
    }

    #[test]
    fn project_keys_edit_and_check_without_a_shell() {
        let mut state = State::new(vec![project("api"), project("web")]);
        state.handle(Key::Right);
        assert_eq!(state.handle(Key::Char('e')), Action::Edit("web".to_owned()));
        assert_eq!(state.handle(Key::Char('d')), Action::Doctor);
        assert_eq!(
            state.handle(Key::Char('t')),
            Action::Appearance(Mode::Light)
        );
        assert_eq!(state.handle(Key::Char('t')), Action::Appearance(Mode::Dark));
        assert_eq!(state.appearance, Mode::Dark);
        assert_eq!(
            state.handle(Key::Char('s')),
            Action::None,
            "f0.app has no way out to a shell"
        );
        assert_eq!(State::new(Vec::new()).handle(Key::Char('e')), Action::None);
    }

    #[test]
    fn any_key_clears_the_message() {
        let mut state = State::new(Vec::new());
        state.error("boom");
        state.handle(Key::Down);
        assert!(state.message.is_none());
    }
}
