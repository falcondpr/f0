use std::fs;
use std::path::{Path, PathBuf};

use serde::de::Error as _;
use serde::{Deserialize, Deserializer};

use crate::error::F0Error;

pub const MAX_TERMINALS: u8 = 6;

pub fn default_branch_prefixes() -> Vec<String> {
    ["feature/", "fix/", "refactor/", "chore/"]
        .map(str::to_owned)
        .to_vec()
}

/// `branch_prefixes` as a list, or the single string older files wrote as `branch_prefix`.
fn one_or_many<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMany {
        One(String),
        Many(Vec<String>),
    }
    Ok(match OneOrMany::deserialize(deserializer)? {
        OneOrMany::One(prefix) => vec![prefix],
        OneOrMany::Many(prefixes) => prefixes,
    })
}

/// A project as written in `~/.config/f0/projects/<name>.toml`. Missing `agent`,
/// `editor` and `layout` fall back to the global defaults.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectFile {
    pub repo: String,
    pub base: Option<String>,
    /// The first one is the default; `f0 new --type` picks another.
    #[serde(
        default = "default_branch_prefixes",
        alias = "branch_prefix",
        deserialize_with = "one_or_many"
    )]
    pub branch_prefixes: Vec<String>,
    pub include_file: Option<String>,
    #[serde(default)]
    pub servers: Vec<Server>,
    pub agent: Option<Agent>,
    pub editor: Option<Editor>,
    pub layout: Option<LayoutConfig>,
}

impl ProjectFile {
    /// The file as written, without the global defaults `Project` fills in.
    pub fn parse(source: &str, path: &Path) -> Result<Self, F0Error> {
        toml::from_str(source).map_err(|e| F0Error::ParseConfig {
            path: path.to_owned(),
            source: Box::new(e),
        })
    }
}

/// `~/.config/f0/config.toml`.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct GlobalFile {
    #[serde(default)]
    defaults: Defaults,
    #[serde(default)]
    keys: Keys,
}

/// tmux keys f0 binds after the prefix. Key tables belong to the tmux server, so they
/// are global, not per project.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields, default)]
pub struct Keys {
    /// Closes the task of the session it is pressed in, like `f0 done`. `None` when
    /// written as `false`.
    #[serde(deserialize_with = "key_or_off")]
    pub done: Option<String>,
}

impl Default for Keys {
    fn default() -> Self {
        Self {
            done: Some("X".to_owned()),
        }
    }
}

/// A tmux key name, or `false` to leave the key alone.
fn key_or_off<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum KeyOrOff {
        Key(String),
        Off(bool),
    }
    match KeyOrOff::deserialize(deserializer)? {
        KeyOrOff::Key(key) if key.is_empty() || key.contains(char::is_whitespace) => {
            Err(D::Error::custom(format!(
                "`{key}` is not a tmux key; write one like \"X\" or `false`"
            )))
        }
        KeyOrOff::Key(key) => Ok(Some(key)),
        KeyOrOff::Off(false) => Ok(None),
        KeyOrOff::Off(true) => Err(D::Error::custom(
            "write the key, like \"X\", or `false` to turn it off",
        )),
    }
}

/// What every workspace uses unless its project says otherwise.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields, default)]
pub struct Defaults {
    pub agent: Agent,
    pub editor: Editor,
    pub layout: LayoutConfig,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Server {
    pub name: String,
    pub cmd: String,
    pub cwd: Option<PathBuf>,
    /// Start with the session. Otherwise the command is typed into its pane and waits
    /// for Enter, unless `--start` names it.
    #[serde(default)]
    pub autostart: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Agent {
    pub cmd: String,
    /// Picks up the agent's last conversation in the worktree when `f0 open` rebuilds a
    /// session.
    #[serde(default)]
    pub resume: Option<String>,
}

impl Agent {
    pub fn new(cmd: impl Into<String>) -> Self {
        Self {
            cmd: cmd.into(),
            resume: None,
        }
    }

    /// The program the pane runs, without arguments or path: `claude`, `codex`.
    pub fn program(&self) -> &str {
        let program = self.cmd.split_whitespace().next().unwrap_or_default();
        program.rsplit('/').next().unwrap_or(program)
    }

    /// `resume`, or `--continue` for Claude Code, which keeps conversations per folder:
    /// every task has its own worktree, so it always finds that task's conversation.
    pub fn resume_cmd(&self) -> Option<String> {
        self.resume
            .clone()
            .or_else(|| (self.program() == "claude").then(|| format!("{} --continue", self.cmd)))
    }
}

impl Default for Agent {
    fn default() -> Self {
        Self::new("claude")
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Editor {
    pub cmd: String,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            cmd: "nvim .".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields, default)]
pub struct LayoutConfig {
    /// Panes in the bottom row. Servers fill them first, the rest are plain shells.
    pub terminals: u8,
    /// Height of the terminal row, in percent of the window.
    pub terminal_height: u8,
    /// Width of the agent column, in percent of the top row.
    pub agent_width: u8,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            terminals: 4,
            terminal_height: 25,
            agent_width: 35,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Project {
    pub name: String,
    pub repo: PathBuf,
    pub base: Option<String>,
    pub branch_prefixes: Vec<String>,
    pub include_file: Option<PathBuf>,
    pub servers: Vec<Server>,
    pub agent: Agent,
    pub editor: Editor,
    pub layout: LayoutConfig,
}

impl Project {
    pub fn from_toml(
        name: &str,
        source: &str,
        path: &Path,
        home: &Path,
        defaults: &Defaults,
    ) -> Result<Self, F0Error> {
        let file = ProjectFile::parse(source, path)?;

        let layout = file.layout.unwrap_or_else(|| defaults.layout.clone());
        validate(name, &file.servers, &layout)?;
        check_branch_prefixes(&file.branch_prefixes).map_err(|reason| F0Error::InvalidProject {
            name: name.to_owned(),
            reason,
        })?;

        Ok(Self {
            name: name.to_owned(),
            repo: expand_home(&file.repo, home),
            base: file.base,
            branch_prefixes: file.branch_prefixes,
            include_file: file.include_file.map(|f| expand_home(&f, home)),
            servers: file.servers,
            agent: file.agent.unwrap_or_else(|| defaults.agent.clone()),
            editor: file.editor.unwrap_or_else(|| defaults.editor.clone()),
            layout,
        })
    }

    /// The file that tells treehouse which ignored files (env files) to copy into a slot:
    /// `include_file` when set, otherwise the repo's committed `.worktreeinclude`.
    pub fn worktree_include(&self) -> PathBuf {
        self.include_file
            .clone()
            .unwrap_or_else(|| self.repo.join(".worktreeinclude"))
    }

    /// Fails while a `<...>` placeholder written by `f0 init` is still in the file.
    pub fn ensure_filled(&self) -> Result<(), F0Error> {
        match self.unfilled_placeholder() {
            Some((key, value)) => Err(F0Error::UnfilledPlaceholder {
                project: self.name.clone(),
                key,
                value: value.to_owned(),
            }),
            None => Ok(()),
        }
    }

    /// Fails when `--start` names a server the project does not have.
    pub fn check_start(&self, start: &[String]) -> Result<(), F0Error> {
        match start
            .iter()
            .find(|name| !self.servers.iter().any(|s| &s.name == *name))
        {
            Some(name) => Err(F0Error::UnknownServer {
                project: self.name.clone(),
                name: name.clone(),
                available: if self.servers.is_empty() {
                    "none".to_owned()
                } else {
                    self.servers
                        .iter()
                        .map(|s| s.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                },
            }),
            None => Ok(()),
        }
    }

    fn unfilled_placeholder(&self) -> Option<(&'static str, &str)> {
        if let Some(base) = self.base.as_deref().filter(|base| is_placeholder(base)) {
            return Some(("base", base));
        }
        self.servers.iter().find_map(|server| {
            if is_placeholder(&server.name) {
                Some(("servers.name", server.name.as_str()))
            } else if is_placeholder(&server.cmd) {
                Some(("servers.cmd", server.cmd.as_str()))
            } else {
                None
            }
        })
    }

    /// The branch `f0 new` creates: the first prefix, or the one `kind` names, written
    /// with or without its trailing `/` (`fix` or `fix/`).
    pub fn branch_for(&self, ticket: &str, kind: Option<&str>) -> Result<String, F0Error> {
        let prefix = match kind {
            None => &self.branch_prefixes[0],
            Some(kind) => self
                .branch_prefixes
                .iter()
                .find(|prefix| *prefix == kind || prefix.trim_end_matches('/') == kind)
                .ok_or_else(|| F0Error::UnknownBranchType {
                    project: self.name.clone(),
                    name: kind.to_owned(),
                    available: self
                        .branch_prefixes
                        .iter()
                        .map(|prefix| prefix.trim_end_matches('/'))
                        .collect::<Vec<_>>()
                        .join(", "),
                })?,
        };
        Ok(format!("{prefix}{ticket}"))
    }

    /// The part of a branch name that identifies the task, used to name its session. The
    /// longest prefix wins, so `fix/` does not hide a configured `fix/ui/`.
    pub fn task_from_branch<'a>(&self, branch: &'a str) -> &'a str {
        self.branch_prefixes
            .iter()
            .filter_map(|prefix| branch.strip_prefix(prefix.as_str()))
            .min_by_key(|task| task.len())
            .unwrap_or(branch)
    }
}

pub fn is_placeholder(value: &str) -> bool {
    value.starts_with('<') && value.ends_with('>')
}

/// A project file `f0 init` is about to write, from your answers or with placeholders.
/// Paths are already shortened with `~/`.
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectDraft {
    pub repo: String,
    pub base: Option<String>,
    /// The base f0 detected, shown as a hint next to `base`.
    pub detected_base: Option<String>,
    pub branch_prefixes: Vec<String>,
    /// `None` when the repo has its own `.worktreeinclude`.
    pub include_file: Option<String>,
    pub servers: Vec<Server>,
    /// Written as `[layout] terminals` when the servers need more panes than the default.
    pub terminals: Option<u8>,
    /// Kept from an existing file by `f0 edit`; `f0 init` leaves them to the defaults.
    pub agent: Option<Agent>,
    pub editor: Option<Editor>,
    pub layout: Option<LayoutConfig>,
}

impl ProjectDraft {
    /// What f0 detects, with `<...>` placeholders that `Project::ensure_filled` rejects
    /// for everything you have to choose.
    pub fn template(
        repo: String,
        include_file: Option<String>,
        detected_base: Option<String>,
    ) -> Self {
        Self {
            repo,
            base: Some("<base branch>".to_owned()),
            detected_base,
            branch_prefixes: default_branch_prefixes(),
            include_file,
            servers: vec![Server {
                name: "<server name>".to_owned(),
                cmd: "<command that starts it>".to_owned(),
                cwd: None,
                autostart: false,
            }],
            terminals: None,
            agent: None,
            editor: None,
            layout: None,
        }
    }

    pub fn render(&self) -> String {
        let has_placeholders = self.base.as_deref().is_some_and(is_placeholder)
            || self
                .servers
                .iter()
                .any(|s| is_placeholder(&s.name) || is_placeholder(&s.cmd));
        let mut out = format!(
            "# f0 project for {}. The file name is the project name.\n",
            self.repo
        );
        out.push_str(if has_placeholders {
            "# Replace every <...> placeholder, then run `f0 doctor`.\n"
        } else {
            "# Run `f0 doctor` after editing it.\n"
        });
        out.push_str(&format!("\nrepo = {}\n\n", toml_string(&self.repo)));

        let hint = self
            .detected_base
            .as_deref()
            .map(|base| format!(" Detected: `{base}`."))
            .unwrap_or_default();
        match &self.base {
            Some(base) => out.push_str(&format!(
                "# Branch new tasks are cut from.{hint}\nbase = {}\n\n",
                toml_string(base)
            )),
            None => out.push_str(&format!(
                "# Branch new tasks are cut from; treehouse infers it when missing.{hint}\n\
                 # base = \"main\"\n\n"
            )),
        }

        let prefixes = self
            .branch_prefixes
            .iter()
            .map(|prefix| toml_string(prefix))
            .collect::<Vec<_>>()
            .join(", ");
        let example = match self.branch_prefixes.as_slice() {
            [first, second, ..] => format!(
                "`f0 new APP-123` creates {first}APP-123; `--type {}` creates {second}APP-123.",
                second.trim_end_matches('/')
            ),
            [first] => format!(
                "`f0 new APP-123` creates {first}APP-123; add prefixes to pick one with `--type`."
            ),
            [] => String::new(),
        };
        out.push_str(&format!("# {example}\nbranch_prefixes = [{prefixes}]\n\n"));

        match &self.include_file {
            Some(path) => out.push_str(&format!(
                "# Ignored files (such as .env) copied into every slot, one pattern per line.\n\
                 # An empty file copies nothing.\n\
                 include_file = {}\n\n",
                toml_string(path)
            )),
            None => out.push_str(
                "# Ignored files copied into every slot come from the repo's .worktreeinclude.\n\n",
            ),
        }

        out.push_str(
            "# One block per dev server, each in its own terminal pane. Its command waits for\n\
             # Enter; `f0 new <ticket> --start <name>` or `autostart = true` starts it with the session.\n",
        );
        if self.servers.is_empty() {
            out.push_str("# [[servers]]\n# name = \"web\"\n# cmd = \"pnpm dev\"\n");
        }
        for (index, server) in self.servers.iter().enumerate() {
            if index > 0 {
                out.push('\n');
            }
            out.push_str(&format!(
                "[[servers]]\nname = {}\ncmd = {}\n",
                toml_string(&server.name),
                toml_string(&server.cmd)
            ));
            if let Some(cwd) = &server.cwd {
                out.push_str(&format!("cwd = {}\n", toml_string(&cwd.to_string_lossy())));
            }
            out.push_str(if server.autostart {
                "autostart = true\n"
            } else {
                "# autostart = true\n"
            });
        }

        if let Some(editor) = &self.editor {
            out.push_str(&format!("\n[editor]\ncmd = {}\n", toml_string(&editor.cmd)));
        }
        if let Some(agent) = &self.agent {
            out.push_str(&format!("\n[agent]\ncmd = {}\n", toml_string(&agent.cmd)));
            if let Some(resume) = &agent.resume {
                out.push_str(&format!("resume = {}\n", toml_string(resume)));
            }
        }
        match (&self.layout, self.terminals) {
            (Some(layout), terminals) => out.push_str(&format!(
                "\n[layout]\nterminals = {}\nterminal_height = {}\nagent_width = {}\n",
                terminals.unwrap_or(layout.terminals),
                layout.terminal_height,
                layout.agent_width
            )),
            (None, Some(terminals)) => {
                out.push_str(&format!("\n[layout]\nterminals = {terminals}\n"))
            }
            (None, None) => {}
        }
        out
    }
}

fn toml_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

/// `path` with the home directory written as `~`, the way project files usually spell it.
pub fn with_tilde(path: &Path, home: &Path) -> String {
    match path.strip_prefix(home) {
        Ok(rest) => format!("~/{}", rest.display()),
        Err(_) => path.display().to_string(),
    }
}

fn validate(name: &str, servers: &[Server], layout: &LayoutConfig) -> Result<(), F0Error> {
    let invalid = |reason: String| F0Error::InvalidProject {
        name: name.to_owned(),
        reason,
    };

    if layout.terminals == 0 || layout.terminals > MAX_TERMINALS {
        return Err(invalid(format!(
            "layout.terminals must be between 1 and {MAX_TERMINALS}"
        )));
    }
    if servers.len() > usize::from(layout.terminals) {
        return Err(invalid(format!(
            "{} servers do not fit in {} terminals; raise layout.terminals",
            servers.len(),
            layout.terminals
        )));
    }
    for (field, value) in [
        ("layout.terminal_height", layout.terminal_height),
        ("layout.agent_width", layout.agent_width),
    ] {
        if !(10..=90).contains(&value) {
            return Err(invalid(format!(
                "{field} must be a percentage between 10 and 90"
            )));
        }
    }
    if let Some(server) = servers
        .iter()
        .find(|s| s.name.trim().is_empty() || s.cmd.trim().is_empty())
    {
        return Err(invalid(format!(
            "server `{}` needs both name and cmd",
            server.name
        )));
    }

    Ok(())
}

/// At least one prefix, none blank or repeated. Returns why not, for `f0 init` and
/// `f0 edit` to ask again.
pub fn check_branch_prefixes(prefixes: &[String]) -> Result<(), String> {
    if prefixes.is_empty() {
        return Err("branch_prefixes needs at least one prefix, like \"feature/\"".to_owned());
    }
    if prefixes.iter().any(|prefix| prefix.trim().is_empty()) {
        return Err("branch_prefixes cannot have an empty prefix".to_owned());
    }
    // `fix` and `fix/` would answer to the same `--type`.
    let kind = |prefix: &str| prefix.trim_end_matches('/').to_owned();
    if let Some((_, prefix)) = prefixes.iter().enumerate().find(|(i, prefix)| {
        prefixes[..*i]
            .iter()
            .any(|earlier| kind(earlier) == kind(prefix))
    }) {
        return Err(format!("branch_prefixes repeats `{prefix}`"));
    }
    Ok(())
}

pub fn config_dir() -> Result<PathBuf, F0Error> {
    if let Some(dir) = std::env::var_os("F0_CONFIG_DIR") {
        return Ok(PathBuf::from(dir));
    }
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(xdg).join("f0"));
    }
    Ok(home_dir()?.join(".config").join("f0"))
}

pub fn home_dir() -> Result<PathBuf, F0Error> {
    dirs::home_dir().ok_or(F0Error::NoHomeDir)
}

pub fn load_defaults(config_dir: &Path) -> Result<Defaults, F0Error> {
    Ok(load_global(config_dir)?.defaults)
}

pub fn load_keys(config_dir: &Path) -> Result<Keys, F0Error> {
    Ok(load_global(config_dir)?.keys)
}

fn load_global(config_dir: &Path) -> Result<GlobalFile, F0Error> {
    let path = config_dir.join("config.toml");
    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(GlobalFile::default()),
        Err(source) => return Err(F0Error::ReadConfig { path, source }),
    };
    parse_global(&source, &path)
}

fn parse_global(source: &str, path: &Path) -> Result<GlobalFile, F0Error> {
    let file: GlobalFile = toml::from_str(source).map_err(|e| F0Error::ParseConfig {
        path: path.to_owned(),
        source: Box::new(e),
    })?;
    validate("defaults", &[], &file.defaults.layout)?;
    Ok(file)
}

pub fn load_projects(config_dir: &Path, defaults: &Defaults) -> Result<Vec<Project>, F0Error> {
    let projects_dir = config_dir.join("projects");
    let entries = match fs::read_dir(&projects_dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            return Err(F0Error::ReadConfig {
                path: projects_dir,
                source,
            });
        }
    };

    let mut paths: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
        .collect();
    paths.sort();

    let home = home_dir()?;
    paths
        .iter()
        .map(|path| {
            let source = fs::read_to_string(path).map_err(|source| F0Error::ReadConfig {
                path: path.clone(),
                source,
            })?;
            let name = path.file_stem().unwrap_or_default().to_string_lossy();
            Project::from_toml(&name, &source, path, &home, defaults)
        })
        .collect()
}

pub fn find_by_name<'a>(
    projects: &'a [Project],
    name: &str,
    config_dir: &Path,
) -> Result<&'a Project, F0Error> {
    projects
        .iter()
        .find(|p| p.name == name)
        .ok_or_else(|| F0Error::UnknownProject {
            name: name.to_owned(),
            config_dir: config_dir.to_owned(),
        })
}

pub fn find_by_repo<'a>(
    projects: &'a [Project],
    repo: &Path,
    config_dir: &Path,
) -> Result<&'a Project, F0Error> {
    project_for_repo(projects, repo).ok_or_else(|| F0Error::ProjectNotFound {
        repo: repo.to_owned(),
        config_dir: config_dir.to_owned(),
    })
}

pub fn project_for_repo<'a>(projects: &'a [Project], repo: &Path) -> Option<&'a Project> {
    let wanted = canonical(repo);
    projects.iter().find(|p| canonical(&p.repo) == wanted)
}

fn canonical(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_owned())
}

fn expand_home(raw: &str, home: &Path) -> PathBuf {
    match raw.strip_prefix("~/") {
        Some(rest) => home.join(rest),
        None if raw == "~" => home.to_owned(),
        None => PathBuf::from(raw),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(source: &str) -> Result<Project, F0Error> {
        Project::from_toml(
            "hubi",
            source,
            Path::new("hubi.toml"),
            Path::new("/home/me"),
            &Defaults::default(),
        )
    }

    #[test]
    fn applies_defaults() {
        let project = parse(r#"repo = "~/dev/hubi""#).unwrap();
        assert_eq!(project.repo, PathBuf::from("/home/me/dev/hubi"));
        assert_eq!(project.base, None);
        assert_eq!(
            project.branch_prefixes,
            ["feature/", "fix/", "refactor/", "chore/"]
        );
        assert_eq!(project.agent.cmd, "claude");
        assert_eq!(project.editor.cmd, "nvim .");
        assert_eq!(project.layout, LayoutConfig::default());
    }

    #[test]
    fn parses_full_project() {
        let project = parse(
            r#"
            repo = "/src/hubi"
            base = "develop"
            include_file = "~/.config/f0/hubi.worktreeinclude"

            [[servers]]
            name = "mobile"
            cwd = "apps/mobile"
            cmd = "npx expo start -c"

            [layout]
            terminals = 3
            "#,
        )
        .unwrap();
        assert_eq!(project.base.as_deref(), Some("develop"));
        assert_eq!(
            project.include_file,
            Some(PathBuf::from("/home/me/.config/f0/hubi.worktreeinclude"))
        );
        assert_eq!(project.servers[0].cwd, Some(PathBuf::from("apps/mobile")));
        assert_eq!(project.layout.terminals, 3);
        assert_eq!(project.layout.agent_width, 35);
    }

    #[test]
    fn servers_wait_unless_autostart_and_start_must_exist() {
        let project = parse(
            r#"
            repo = "/src/hubi"
            [[servers]]
            name = "nexus"
            cmd = "pnpm nx serve @org/nexus"
            autostart = true
            [[servers]]
            name = "admin"
            cmd = "pnpm nx dev @org/admin"
            "#,
        )
        .unwrap();
        assert!(project.servers[0].autostart);
        assert!(!project.servers[1].autostart);
        project.check_start(&["admin".to_owned()]).unwrap();
        let err = project
            .check_start(&["web".to_owned()])
            .unwrap_err()
            .to_string();
        assert!(err.contains("no server `web`"), "{err}");
        assert!(err.contains("nexus, admin"), "{err}");
    }

    #[test]
    fn rejects_more_servers_than_terminals() {
        let err = parse(
            r#"
            repo = "/src/hubi"
            [layout]
            terminals = 1
            [[servers]]
            name = "a"
            cmd = "a"
            [[servers]]
            name = "b"
            cmd = "b"
            "#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("do not fit"), "{err}");
    }

    #[test]
    fn rejects_unknown_fields() {
        assert!(parse("repo = \"/x\"\nservrs = []").is_err());
    }

    #[test]
    fn branch_type_picks_the_prefix_and_defaults_to_the_first() {
        let project = parse(r#"repo = "/x""#).unwrap();
        assert_eq!(project.branch_for("HB-1", None).unwrap(), "feature/HB-1");
        assert_eq!(project.branch_for("HB-1", Some("fix")).unwrap(), "fix/HB-1");
        assert_eq!(
            project.branch_for("HB-1", Some("chore/")).unwrap(),
            "chore/HB-1"
        );
        let err = project
            .branch_for("HB-1", Some("feat"))
            .unwrap_err()
            .to_string();
        assert!(err.contains("no branch type `feat`"), "{err}");
        assert!(err.contains("feature, fix, refactor, chore"), "{err}");
    }

    #[test]
    fn task_strips_any_configured_prefix_the_longest_first() {
        let project = parse(
            r#"repo = "/x"
            branch_prefixes = ["feature/", "fix/", "fix/ui/"]"#,
        )
        .unwrap();
        assert_eq!(project.task_from_branch("feature/HB-1"), "HB-1");
        assert_eq!(project.task_from_branch("fix/HB-2"), "HB-2");
        assert_eq!(project.task_from_branch("fix/ui/HB-3"), "HB-3");
        assert_eq!(project.task_from_branch("chore/HB-4"), "chore/HB-4");
    }

    #[test]
    fn single_branch_prefix_still_loads() {
        let project = parse("repo = \"/x\"\nbranch_prefix = \"feat/\"").unwrap();
        assert_eq!(project.branch_prefixes, ["feat/"]);
        assert_eq!(project.branch_for("X-1", None).unwrap(), "feat/X-1");
        assert!(project.branch_for("X-1", Some("fix")).is_err());
        assert!(
            parse("repo = \"/x\"\nbranch_prefix = \"a/\"\nbranch_prefixes = [\"b/\"]").is_err()
        );
    }

    #[test]
    fn rejects_empty_blank_or_repeated_prefixes() {
        for prefixes in [r#"[]"#, r#"["feature/", " "]"#, r#"["fix", "fix/"]"#] {
            let err = parse(&format!("repo = \"/x\"\nbranch_prefixes = {prefixes}"))
                .unwrap_err()
                .to_string();
            assert!(err.contains("branch_prefixes"), "{prefixes}: {err}");
        }
    }

    #[test]
    fn project_falls_back_to_global_defaults() {
        let defaults = parse_global(
            "[defaults.editor]\ncmd = \"hx .\"\n[defaults.layout]\nterminals = 2",
            Path::new("config.toml"),
        )
        .unwrap()
        .defaults;
        let project = Project::from_toml(
            "hubi",
            "repo = \"/x\"\n[agent]\ncmd = \"codex\"",
            Path::new("hubi.toml"),
            Path::new("/home/me"),
            &defaults,
        )
        .unwrap();
        assert_eq!(project.editor.cmd, "hx .");
        assert_eq!(project.agent.cmd, "codex");
        assert_eq!(project.layout.terminals, 2);
        assert_eq!(project.layout.agent_width, 35);
    }

    #[test]
    fn defaults_reject_project_keys_and_bad_layouts() {
        let path = Path::new("config.toml");
        assert!(parse_global("[defaults]\nrepo = \"/x\"", path).is_err());
        assert!(parse_global("[defaults.layout]\nterminals = 0", path).is_err());
        assert_eq!(
            parse_global("", path).unwrap().defaults,
            Defaults::default()
        );
    }

    #[test]
    fn done_key_defaults_to_x_and_turns_off_with_false() {
        let path = Path::new("config.toml");
        let keys = |source: &str| parse_global(source, path).map(|file| file.keys.done);
        assert_eq!(keys("").unwrap().as_deref(), Some("X"));
        assert_eq!(
            keys("[keys]\ndone = \"M-x\"").unwrap().as_deref(),
            Some("M-x")
        );
        assert_eq!(keys("[keys]\ndone = false").unwrap(), None);
        for bad in [
            "done = true",
            "done = \"\"",
            "done = \"C-a X\"",
            "close = \"X\"",
        ] {
            assert!(keys(&format!("[keys]\n{bad}")).is_err(), "{bad}");
        }
    }

    #[test]
    fn template_parses_and_keeps_its_placeholders() {
        let source = ProjectDraft::template(
            "~/dev/my \"app\"".to_owned(),
            Some("~/.config/f0/app.worktreeinclude".to_owned()),
            Some("main".to_owned()),
        )
        .render();
        assert!(source.contains("Detected: `main`"), "{source}");
        assert!(
            source.contains("Replace every <...> placeholder"),
            "{source}"
        );
        let project = parse(&source).unwrap();
        assert_eq!(project.repo, PathBuf::from("/home/me/dev/my \"app\""));
        assert_eq!(
            project.include_file,
            Some(PathBuf::from("/home/me/.config/f0/app.worktreeinclude"))
        );
        let err = project.ensure_filled().unwrap_err().to_string();
        assert!(err.contains("`<base branch>` in `base`"), "{err}");

        let filled = source
            .replace("<base branch>", "main")
            .replace("<server name>", "web")
            .replace("<command that starts it>", "bun dev");
        parse(&filled).unwrap().ensure_filled().unwrap();
    }

    #[test]
    fn answered_draft_renders_a_complete_project() {
        let draft = ProjectDraft {
            repo: "/src/app".to_owned(),
            base: None,
            detected_base: None,
            branch_prefixes: vec!["feat/".to_owned(), "bug/".to_owned()],
            include_file: None,
            servers: (1..=5)
                .map(|i| Server {
                    name: format!("s{i}"),
                    cmd: format!("run {i}"),
                    cwd: (i == 2).then(|| PathBuf::from("apps/web")),
                    autostart: i == 1,
                })
                .collect(),
            terminals: Some(6),
            agent: Some(Agent {
                cmd: "codex".to_owned(),
                resume: Some("codex resume --last".to_owned()),
            }),
            editor: None,
            layout: Some(LayoutConfig {
                terminals: 4,
                terminal_height: 30,
                agent_width: 40,
            }),
        };
        let source = draft.render();
        assert!(!source.contains("placeholder"), "{source}");
        assert!(
            source.contains("`--type bug` creates bug/APP-123"),
            "{source}"
        );
        let project = parse(&source).unwrap();
        project.ensure_filled().unwrap();
        assert_eq!(project.base, None);
        assert_eq!(project.include_file, None);
        assert_eq!(project.branch_for("X-1", None).unwrap(), "feat/X-1");
        assert_eq!(project.branch_for("X-1", Some("bug")).unwrap(), "bug/X-1");
        assert_eq!(project.servers, draft.servers);
        assert_eq!(project.layout.terminals, 6);
        assert_eq!(project.layout.terminal_height, 30);
        assert_eq!(project.agent, draft.agent.clone().unwrap());
        assert_eq!(project.editor.cmd, "nvim .");
    }

    #[test]
    fn draft_without_servers_still_parses() {
        let mut draft = ProjectDraft::template("/src/app".to_owned(), None, None);
        draft.base = Some("main".to_owned());
        draft.servers.clear();
        let project = parse(&draft.render()).unwrap();
        assert!(project.servers.is_empty());
        project.ensure_filled().unwrap();
    }

    #[test]
    fn with_tilde_shortens_only_paths_under_home() {
        let home = Path::new("/home/me");
        assert_eq!(with_tilde(Path::new("/home/me/dev/app"), home), "~/dev/app");
        assert_eq!(with_tilde(Path::new("/srv/app"), home), "/srv/app");
    }

    #[test]
    fn agent_program_drops_arguments_and_path() {
        assert_eq!(Agent::new("claude").program(), "claude");
        assert_eq!(Agent::new("codex --full-auto").program(), "codex");
        assert_eq!(Agent::new("/opt/bin/opencode run").program(), "opencode");
    }

    #[test]
    fn claude_resumes_by_default_and_other_agents_only_when_set() {
        assert_eq!(
            Agent::new("claude --model opus").resume_cmd().as_deref(),
            Some("claude --model opus --continue")
        );
        assert_eq!(Agent::new("codex").resume_cmd(), None);

        let project =
            parse("repo = \"/x\"\n[agent]\ncmd = \"codex\"\nresume = \"codex resume --last\"")
                .unwrap();
        assert_eq!(
            project.agent.resume_cmd().as_deref(),
            Some("codex resume --last")
        );
    }

    #[test]
    fn worktree_include_defaults_to_the_repo_file() {
        let project = parse(r#"repo = "/src/hubi""#).unwrap();
        assert_eq!(
            project.worktree_include(),
            PathBuf::from("/src/hubi/.worktreeinclude")
        );

        let project = parse("repo = \"/src/hubi\"\ninclude_file = \"/etc/hubi.include\"").unwrap();
        assert_eq!(
            project.worktree_include(),
            PathBuf::from("/etc/hubi.include")
        );
    }
}
