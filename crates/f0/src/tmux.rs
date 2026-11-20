use std::fs::File;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::error::F0Error;
use crate::layout::{Direction, Layout, Pane};
use crate::process;

const PATH_OPTION: &str = "@f0_path";
/// Set on the panes of agents, with the command of each.
const AGENT_OPTION: &str = "@f0_agent";
const AGENT_PROMPT: &str = "Agent (Enter for the project's): ";
const DEFAULT_SIZE: (u16, u16) = (240, 60);

/// A key f0 binds after the prefix, for the f0 session it is pressed in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum F0Key {
    /// Closes the task, like `f0 done`.
    Done,
    /// Starts another agent, like `f0 agent`.
    Agent,
}

impl F0Key {
    /// Its name under `[keys]` in config.toml.
    pub fn name(self) -> &'static str {
        match self {
            F0Key::Done => "done",
            F0Key::Agent => "agent",
        }
    }

    /// The key f0 last bound for it, kept on the tmux server so a changed or turned off
    /// key can be unbound.
    fn option(self) -> &'static str {
        match self {
            F0Key::Done => "@f0_done_key",
            F0Key::Agent => "@f0_agent_key",
        }
    }

    fn command(self, exe: &str) -> String {
        match self {
            F0Key::Done => done_command(exe),
            F0Key::Agent => agent_command(exe),
        }
    }

    /// Whether `binding`, as `list-keys` prints it, runs this key's f0 command.
    fn is_bound_in(self, binding: &str) -> bool {
        is_f0_binding(binding)
            && match self {
                F0Key::Done => binding.contains(" done ") || binding.contains(" close-menu "),
                F0Key::Agent => binding.contains(" agent "),
            }
    }
}

/// An agent pane of a running session.
#[derive(Debug, Clone, PartialEq)]
pub struct AgentPane {
    pub id: String,
    /// `None` for an agent pane of a session an older f0 built, known only by its title.
    pub cmd: Option<String>,
    pub top: u16,
    pub height: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    pub name: String,
    pub path: PathBuf,
    /// When a client last used the session, in seconds since the Unix epoch.
    pub activity: Option<u64>,
}

/// What `f0 new` and `f0 open` do with the session once it is ready.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Attach {
    /// Print the session name and leave it running in the background.
    No,
    /// Replace this process with `tmux attach`, as the CLI does.
    Replace,
    /// Run `tmux attach` as a child and return when it detaches or the session ends, so
    /// the launcher can show its list again.
    Wait,
}

pub fn session_name(project: &str, task: &str) -> String {
    sanitize(&format!("{project}/{task}"))
}

/// tmux treats `.` and `:` as target separators, so they cannot appear in session names.
pub fn sanitize(name: &str) -> String {
    name.replace(['.', ':'], "_")
}

pub fn has_session(name: &str) -> bool {
    process::output("tmux", ["has-session", "-t", &exact(name)], None).is_ok()
}

/// Sessions created by f0, identified by the worktree path stored on them.
pub fn sessions() -> Result<Vec<Session>, F0Error> {
    let format = format!("#{{session_name}}\t#{{{PATH_OPTION}}}\t#{{session_activity}}");
    match process::output("tmux", ["list-sessions", "-F", &format], None) {
        Ok(out) => Ok(parse_sessions(&out)),
        Err(F0Error::CommandFailed { stderr, .. }) if is_no_server(&stderr) => Ok(Vec::new()),
        Err(e) => Err(e),
    }
}

pub fn sessions_for(path: &Path) -> Result<Vec<Session>, F0Error> {
    Ok(sessions()?.into_iter().filter(|s| s.path == path).collect())
}

/// Session that owns the pane this process runs in, if any.
pub fn current_session() -> Option<String> {
    let pane = std::env::var("TMUX_PANE").ok()?;
    process::output(
        "tmux",
        ["display-message", "-p", "-t", &pane, "#{session_name}"],
        None,
    )
    .ok()
}

/// Asks in the status line of the client showing `session`, then runs `template` with
/// `run-shell -b`, which reads `args` as `#{1}`, `#{2}`... The job belongs to the tmux
/// server, so it outlives every pane of the session. `confirm-before` would take only a
/// lowercase `y`, so the prompt reads one key and the job checks it.
pub fn confirm_then_run(
    session: &str,
    prompt: &str,
    template: &str,
    args: &[&str],
) -> Result<(), F0Error> {
    let client = client_of(session)?;
    tmux(&[
        "command-prompt",
        "-1",
        "-t",
        &client,
        "-p",
        &prompt.replace('#', "##"),
        &confirmed_job(template, args),
    ])
    .map(drop)
}

/// The job of `confirm_then_run`: tmux puts the key pressed in place of `%1`, the last
/// argument, and anything but `y` or `Y` runs nothing.
fn confirmed_job(template: &str, args: &[&str]) -> String {
    let answer = args.len() + 1;
    let guarded = format!("case #{{q:{answer}}} in y|Y) {template};; esac");
    let mut args = args.to_vec();
    args.push("%1");
    run_shell(&guarded, &args)
}

/// One choice of `menu`: label, key, and the job it runs like `confirm_then_run`.
pub struct MenuItem<'a> {
    pub label: &'a str,
    pub key: &'a str,
    pub template: &'a str,
    pub args: &'a [&'a str],
}

/// Shows a menu in the middle of the client showing `session`. It does not wait for
/// the choice; Esc or `q` closes it without running anything.
pub fn menu(session: &str, title: &str, items: &[MenuItem]) -> Result<(), F0Error> {
    let client = client_of(session)?;
    let title = format!("#[align=centre] {} ", title.replace('#', "##"));
    let jobs: Vec<String> = items
        .iter()
        .map(|item| background_job(item.template, item.args))
        .collect();
    let mut command = vec![
        "display-menu",
        "-c",
        &client,
        "-T",
        &title,
        "-x",
        "C",
        "-y",
        "C",
    ];
    for (item, job) in items.iter().zip(&jobs) {
        command.extend([item.label, item.key, job.as_str()]);
    }
    tmux(&command).map(drop)
}

/// The client showing `session`, which `confirm_then_run` and `menu` ask in.
pub fn client_of(session: &str) -> Result<String, F0Error> {
    let clients = tmux(&[
        "list-clients",
        "-t",
        &exact(session),
        "-F",
        "#{client_name}",
    ])?;
    clients
        .lines()
        .next()
        .map(str::to_owned)
        .ok_or_else(|| F0Error::NoClient {
            session: session.to_owned(),
        })
}

/// `run-shell -b` of `template` with `args`, for `display-menu`, which expands formats
/// in the command before running it. That would empty `#{1}` before `run-shell` sees
/// its arguments, so every `#` is doubled.
fn background_job(template: &str, args: &[&str]) -> String {
    run_shell(template, args).replace('#', "##")
}

fn run_shell(template: &str, args: &[&str]) -> String {
    std::iter::once("run-shell -b".to_owned())
        .chain(
            std::iter::once(template)
                .chain(args.iter().copied())
                .map(quote),
        )
        .collect::<Vec<_>>()
        .join(" ")
}

/// Runs `template` with `run-shell -b` right away, like `confirm_then_run` once
/// confirmed.
pub fn run_in_background(template: &str, args: &[&str]) -> Result<(), F0Error> {
    let mut command = vec!["run-shell", "-b", template];
    command.extend(args);
    tmux(&command).map(drop)
}

pub fn server_running() -> bool {
    tmux(&["list-sessions"]).is_ok()
}

/// Runs `script` (config/tmux/appearance.sh) on the running server right away, instead
/// of on its next status refresh. Unsetting `@f0-appearance` makes it repaint even when
/// it believes nothing changed.
pub fn refresh_appearance(script: &Path) -> Result<(), F0Error> {
    let script = shell_quote(&script.to_string_lossy());
    tmux(&[
        "set-option",
        "-gu",
        "@f0-appearance",
        ";",
        "run-shell",
        "-b",
        &script,
    ])
    .map(drop)
}

/// The key tmux waits for before a bound key, like `C-a`.
pub fn prefix() -> Result<String, F0Error> {
    tmux(&["show-options", "-gv", "prefix"])
}

/// Binds `key`, after the prefix, to what `kind` does in the f0 session it is pressed
/// in, and unbinds the key an earlier f0 bound for `kind` when it is a different one.
/// `None` only unbinds. It never replaces a key bound to something other than f0.
pub fn bind_key(kind: F0Key, key: Option<&str>, exe: &str) -> Result<(), F0Error> {
    let previous = tmux(&["show-options", "-gqv", kind.option()])?;
    if !previous.is_empty()
        && Some(previous.as_str()) != key
        && prefix_binding(&previous)?.is_some_and(|binding| kind.is_bound_in(&binding))
    {
        tmux(&["unbind-key", "-T", "prefix", &previous])?;
    }
    let Some(key) = key else {
        return tmux(&["set-option", "-gu", kind.option()]).map(drop);
    };
    if let Some(binding) = prefix_binding(key)?
        && !is_f0_binding(&binding)
    {
        return Err(F0Error::KeyTaken {
            name: kind.name(),
            key: key.to_owned(),
            binding,
        });
    }
    let run = kind.command(exe);
    tmux(&[
        "bind-key",
        "-T",
        "prefix",
        key,
        "if-shell",
        "-F",
        &format!("#{{{PATH_OPTION}}}"),
        &run,
        "display-message \"Not an f0 session\"",
    ])
    .map_err(|e| match e {
        F0Error::CommandFailed { stderr, .. } if stderr.starts_with("unknown key") => {
            F0Error::InvalidKey {
                name: kind.name(),
                key: key.to_owned(),
            }
        }
        e => e,
    })?;
    tmux(&["set-option", "-g", kind.option(), key]).map(drop)
}

/// What `key` does after the prefix, as `list-keys` prints it; `None` when unbound.
pub fn prefix_binding(key: &str) -> Result<Option<String>, F0Error> {
    match tmux(&["list-keys", "-T", "prefix", key]) {
        Ok(binding) => Ok(Some(binding)),
        Err(F0Error::CommandFailed { stderr, .. })
            if stderr.starts_with("unknown key") || stderr.starts_with("invalid key") =>
        {
            Ok(None)
        }
        Err(e) => Err(e),
    }
}

/// Binding from `bind_key`, or from the `bind X` block older f0 tmux configs had, which
/// ran `f0 done`.
pub fn is_f0_binding(binding: &str) -> bool {
    binding.contains(PATH_OPTION)
        && (binding.contains(" done ")
            || binding.contains(" close-menu ")
            || binding.contains(" agent "))
}

/// Puts `keys` at the start of the status line of `session`, before the global
/// `status-right`, which gets that much more room so its end is not cut. Built from the
/// global values every time, so applying it again does not repeat it. No keys bring back
/// the global status line.
pub fn show_key_hints(session: &str, keys: &[(F0Key, &str)]) -> Result<(), F0Error> {
    let target = format!("{}:", exact(session));
    if keys.is_empty() {
        tmux(&["set-option", "-u", "-t", &target, "status-right"])?;
        return tmux(&["set-option", "-u", "-t", &target, "status-right-length"]).map(drop);
    }
    let status = process::line_output("tmux", ["show-options", "-gv", "status-right"], None)?;
    let length: usize = tmux(&["show-options", "-gv", "status-right-length"])?
        .parse()
        .unwrap_or_default();
    let hint = key_hints(&prefix()?, keys);
    tmux(&[
        "set-option",
        "-t",
        &target,
        "status-right",
        &format!("#[dim]{}#[default]{status}", hint.replace('#', "##")),
    ])?;
    tmux(&[
        "set-option",
        "-t",
        &target,
        "status-right-length",
        &(length + hint.chars().count()).to_string(),
    ])
    .map(drop)
}

fn key_hints(prefix: &str, keys: &[(F0Key, &str)]) -> String {
    keys.iter()
        .map(|(kind, key)| {
            let what = match kind {
                F0Key::Done => "close task",
                F0Key::Agent => "new agent",
            };
            format!("{prefix} {key} {what}  ")
        })
        .collect()
}

/// The command a pressed key runs: f0 shows the menu of ways to close the task.
/// `TMUX_PANE` tells f0 which session it is closing,
/// since `run-shell` jobs do not run inside a pane. tmux shows only the stdout of the
/// job, so errors go there too. The full path of f0 because the PATH
/// of the tmux server may not have it.
fn done_command(exe: &str) -> String {
    let job = format!(
        "TMUX_PANE=#{{pane_id}} {} close-menu #{{q:{PATH_OPTION}}} 2>&1",
        shell_quote(&exe.replace('#', "##"))
    );
    format!("run-shell -b {}", quote(&job))
}

/// The command a pressed key runs to start another agent: it asks for one in the status
/// line, then runs `f0 agent` with the answer for the pane the key was pressed in.
/// `%%%` is the answer with its quotes escaped for tmux; the shell splits it into words,
/// which `f0 agent` joins back. An empty answer starts the project's agent.
fn agent_command(exe: &str) -> String {
    let job = format!(
        "TMUX_PANE=#{{pane_id}} {} agent -- %%% 2>&1",
        shell_quote(&exe.replace('#', "##"))
    );
    format!(
        "command-prompt -p {} {}",
        quote(AGENT_PROMPT),
        quote(&format!("run-shell -b {}", quote(&job)))
    )
}

/// A shell command line that runs `args` in the terminal of a client that left tmux, and
/// on failure keeps `failure` and the error on screen until Enter, since that terminal
/// may close as soon as the command ends. tmux runs it with the `default-shell`, which
/// may not be a POSIX shell, hence `/bin/sh`.
pub fn terminal_command(args: &[&str], failure: &str) -> String {
    let command = args
        .iter()
        .map(|arg| shell_quote(arg))
        .collect::<Vec<_>>()
        .join(" ");
    let script = format!(
        "{command} || {{ echo; echo {}; read -r _; }}",
        shell_quote(failure)
    );
    format!("/bin/sh -c {}", shell_quote(&script))
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', r"'\''"))
}

/// `value` as one argument of a tmux command string, such as the command a
/// prompt or menu runs. Inside double quotes tmux expands `$VAR` and escapes.
fn quote(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for c in value.chars() {
        if matches!(c, '"' | '\\' | '$') {
            out.push('\\');
        }
        out.push(c);
    }
    out.push('"');
    out
}

/// Hook that restarts the shell of the pane it runs in and types `command` in the new
/// prompt. Hook arguments get no format expansion, so the command goes in literally;
/// single quotes also keep tmux from expanding a leading `~`.
fn restart_hook(command: Option<&str>) -> String {
    match command {
        Some(command) => format!("respawn-pane ; send-keys -l {}", shell_quote(command)),
        None => "respawn-pane".to_owned(),
    }
}

pub fn create_session(name: &str, worktree: &Path, layout: &Layout) -> Result<(), F0Error> {
    let root = layout.panes[0].cwd.to_string_lossy();
    let env: Vec<String> = layout.env.iter().map(|(k, v)| format!("{k}={v}")).collect();
    let mut args = vec![
        "new-session",
        "-d",
        "-P",
        "-F",
        "#{pane_id}",
        "-s",
        name,
        "-n",
        "main",
        "-c",
        &root,
    ];
    // Splits are percentages of the size the session is born with; building it at the
    // final size keeps them exact instead of distorted by the resize on attach.
    let (width, height) = client_size().unwrap_or(DEFAULT_SIZE);
    let (width, height) = (width.to_string(), height.to_string());
    args.extend(["-x", &width, "-y", &height]);
    for var in &env {
        args.extend(["-e", var.as_str()]);
    }
    let first = tmux(&args)?;

    tmux(&[
        "set-option",
        "-t",
        &format!("{}:", exact(name)),
        PATH_OPTION,
        &worktree.to_string_lossy(),
    ])?;

    let mut ids = vec![String::new(); layout.panes.len()];
    ids[0] = first;
    for split in &layout.splits {
        let flag = match split.direction {
            Direction::Right => "-h",
            Direction::Below => "-v",
        };
        let size = format!("{}%", split.percent);
        let cwd = layout.panes[split.new].cwd.to_string_lossy();
        ids[split.new] = tmux(&[
            "split-window",
            flag,
            "-l",
            &size,
            "-c",
            &cwd,
            "-t",
            &ids[split.target],
            "-P",
            "-F",
            "#{pane_id}",
        ])?;
    }

    for (pane, id) in layout.panes.iter().zip(&ids) {
        setup_pane(id, pane)?;
    }

    tmux(&["select-pane", "-t", &ids[layout.focus]])?;
    Ok(())
}

/// Titles the pane `id`, keeps it on exit, and types its command.
fn setup_pane(id: &str, pane: &Pane) -> Result<(), F0Error> {
    // Agents like Claude Code retitle their pane with escape sequences; the title
    // f0 sets is what tells the panes apart. tmux before 3.4 lacks the option.
    let _ = tmux(&["set-option", "-p", "-t", id, "allow-set-title", "off"]);
    // Uppercase like the panel titles of an editor; tmux formats cannot change case.
    tmux(&["select-pane", "-t", id, "-T", &pane.title.to_uppercase()])?;
    if let Some(agent) = &pane.agent {
        tmux(&["set-option", "-p", "-t", id, AGENT_OPTION, agent])?;
    }
    // Ctrl-d at an empty prompt would otherwise close the pane and collapse the
    // layout; `kill-pane` still closes it on purpose. `pane-died` only fires on panes
    // that remain on exit.
    tmux(&["set-option", "-p", "-t", id, "remain-on-exit", "on"])?;
    tmux(&[
        "set-hook",
        "-p",
        "-t",
        id,
        "pane-died",
        &restart_hook(pane.restart.as_deref()),
    ])?;
    if let Some(command) = &pane.command {
        tmux(&["send-keys", "-t", id, "-l", command])?;
        if pane.run {
            tmux(&["send-keys", "-t", id, "Enter"])?;
        }
    }
    Ok(())
}

/// The agent panes of `session`, top to bottom.
pub fn agent_panes(session: &str) -> Result<Vec<AgentPane>, F0Error> {
    let format = format!(
        "#{{pane_id}}\t#{{pane_top}}\t#{{pane_height}}\t#{{{AGENT_OPTION}}}\t#{{pane_title}}"
    );
    let out = process::line_output(
        "tmux",
        ["list-panes", "-s", "-t", &exact(session), "-F", &format],
        None,
    )?;
    Ok(parse_agent_panes(&out))
}

/// Starts `pane` under the agents of `session`, gives every agent the same height, and
/// moves to it.
pub fn add_agent(session: &str, pane: &Pane) -> Result<(), F0Error> {
    let last = agent_panes(session)?
        .pop()
        .ok_or_else(|| F0Error::NoAgentPane {
            session: session.to_owned(),
        })?;
    let cwd = pane.cwd.to_string_lossy();
    let id = tmux(&[
        "split-window",
        "-v",
        "-c",
        &cwd,
        "-t",
        &last.id,
        "-P",
        "-F",
        "#{pane_id}",
    ])?;
    setup_pane(&id, pane)?;

    let agents = agent_panes(session)?;
    let total = agents.iter().map(|a| usize::from(a.height)).sum::<usize>() + agents.len() - 1;
    // Top to bottom, each resize gives rows to or takes them from the pane under it, so
    // the last one ends up with what is left.
    let heights = even_heights(total, agents.len());
    for (agent, height) in agents.iter().zip(heights).take(agents.len() - 1) {
        tmux(&["resize-pane", "-t", &agent.id, "-y", &height.to_string()])?;
    }
    tmux(&["select-pane", "-t", &id]).map(drop)
}

/// Heights for `count` panes stacked in `total` rows, with a border row between each
/// pair; the top panes take the rows left over.
fn even_heights(total: usize, count: usize) -> Vec<usize> {
    if count == 0 {
        return Vec::new();
    }
    let rows = total.saturating_sub(count - 1);
    (0..count)
        .map(|index| rows / count + usize::from(index < rows % count))
        .collect()
}

/// Panes marked with `AGENT_OPTION`, or titled like an agent by an older f0, sorted top
/// to bottom.
fn parse_agent_panes(out: &str) -> Vec<AgentPane> {
    let mut panes: Vec<AgentPane> = out
        .lines()
        .filter_map(|line| {
            let mut fields = line.splitn(5, '\t');
            let (id, top, height) = (fields.next()?, fields.next()?, fields.next()?);
            let cmd = fields.next().unwrap_or_default();
            let title = fields.next().unwrap_or_default();
            if cmd.is_empty() && !title.starts_with("AGENT: ") {
                return None;
            }
            Some(AgentPane {
                id: id.to_owned(),
                cmd: (!cmd.is_empty()).then(|| cmd.to_owned()),
                top: top.parse().ok()?,
                height: height.parse().ok()?,
            })
        })
        .collect();
    panes.sort_by_key(|pane| pane.top);
    panes
}

pub fn kill_session(name: &str) -> Result<(), F0Error> {
    tmux(&["kill-session", "-t", &exact(name)]).map(drop)
}

/// Moves the current client to `name`, or attaches this terminal to it when running
/// outside tmux: replacing this process with `tmux attach`, or with `Attach::Wait`
/// waiting for it to end. `Attach::No` does nothing.
pub fn attach(name: &str, mode: Attach) -> Result<(), F0Error> {
    let target = exact(name);
    if mode == Attach::No {
        return Ok(());
    }
    if std::env::var_os("TMUX").is_some() {
        return tmux(&["switch-client", "-t", &target]).map(drop);
    }
    let mut command = Command::new("tmux");
    command.args(["attach-session", "-t", &target]);
    if mode == Attach::Replace {
        let source = command.exec();
        return Err(F0Error::Spawn {
            program: "tmux".to_owned(),
            source,
        });
    }
    // Detaching, `f0 stop` and `f0 done` all end the client with success.
    let status = command.status().map_err(|source| F0Error::Spawn {
        program: "tmux".to_owned(),
        source,
    })?;
    if !status.success() {
        return Err(F0Error::CommandFailed {
            command: format!("tmux attach-session -t {target}"),
            stderr: format!("exited with {status}"),
        });
    }
    Ok(())
}

fn tmux(args: &[&str]) -> Result<String, F0Error> {
    process::output("tmux", args, None)
}

/// Size of the terminal that will show the session: the tmux client when running
/// inside tmux, otherwise the controlling terminal.
fn client_size() -> Option<(u16, u16)> {
    if std::env::var_os("TMUX").is_some() {
        let out = process::output(
            "tmux",
            ["display-message", "-p", "#{client_width} #{client_height}"],
            None,
        )
        .ok()?;
        return parse_size(&out);
    }

    let tty = File::open("/dev/tty").ok()?;
    let out = Command::new("stty")
        .arg("size")
        .stdin(tty)
        .stderr(Stdio::null())
        .output()
        .ok()?;
    // stty prints rows first.
    let (rows, cols) = parse_size(&String::from_utf8_lossy(&out.stdout))?;
    Some((cols, rows))
}

fn parse_size(out: &str) -> Option<(u16, u16)> {
    let (width, height) = out.trim().split_once(' ')?;
    Some((width.parse().ok()?, height.parse().ok()?))
}

/// `=` makes tmux match the session name exactly instead of by prefix.
fn exact(name: &str) -> String {
    format!("={name}")
}

fn is_no_server(stderr: &str) -> bool {
    stderr.contains("no server running") || stderr.contains("error connecting to")
}

fn parse_sessions(out: &str) -> Vec<Session> {
    out.lines()
        .filter_map(|line| {
            let mut fields = line.split('\t');
            let (name, path) = (fields.next()?, fields.next()?);
            let activity = fields.next().and_then(|field| field.parse().ok());
            (!path.is_empty()).then(|| Session {
                name: name.to_owned(),
                path: PathBuf::from(path),
                activity,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_name_avoids_tmux_separators() {
        assert_eq!(session_name("hubi", "HB-1"), "hubi/HB-1");
        assert_eq!(session_name("hubi", "v1.2:fix"), "hubi/v1_2_fix");
    }

    #[test]
    fn quote_keeps_values_literal_for_tmux() {
        assert_eq!(quote("/a b/f0"), r#""/a b/f0""#);
        assert_eq!(quote(r#"it's "x" $HOME \n"#), r#""it's \"x\" \$HOME \\n""#);
    }

    #[test]
    fn restart_hook_types_the_command_literally() {
        assert_eq!(restart_hook(None), "respawn-pane");
        assert_eq!(
            restart_hook(Some(r#"~/x "@a/b" it's $HOME #{pane_id}"#)),
            r#"respawn-pane ; send-keys -l '~/x "@a/b" it'\''s $HOME #{pane_id}'"#
        );
    }

    #[test]
    fn confirmed_job_runs_only_on_yes_in_either_case() {
        assert_eq!(
            confirmed_job("#{q:1} done #{q:2}", &["/bin/f0", "/pool/1"]),
            "run-shell -b \"case #{q:3} in y|Y) #{q:1} done #{q:2};; esac\" \"/bin/f0\" \"/pool/1\" \"%1\""
        );
    }

    #[test]
    fn background_job_survives_the_format_expansion_of_display_menu() {
        assert_eq!(
            background_job("#{q:1} done #{q:2}", &["/bin/f0", "/pool/#1"]),
            "run-shell -b \"##{q:1} done ##{q:2}\" \"/bin/f0\" \"/pool/##1\""
        );
    }

    #[test]
    fn done_command_runs_this_f0_for_the_pressed_pane() {
        assert_eq!(
            done_command("/a b/it's/f0"),
            r##"run-shell -b "TMUX_PANE=#{pane_id} '/a b/it'\\''s/f0' close-menu #{q:@f0_path} 2>&1""##
        );
    }

    #[test]
    fn terminal_command_keeps_args_literal_and_waits_on_failure() {
        let run = |line: String| {
            let out = Command::new("sh")
                .args(["-c", &line])
                .stdin(Stdio::null())
                .output()
                .unwrap();
            String::from_utf8(out.stdout).unwrap()
        };
        let arg = "/a b/it's #1 $HOME `x` \\";
        assert_eq!(
            run(terminal_command(&["printf", "%s|", arg], "Failed.")),
            format!("{arg}|")
        );
        assert_eq!(
            run(terminal_command(&["false"], "it's failed")),
            "\nit's failed\n"
        );
    }

    #[test]
    fn recognizes_f0_bindings_old_and_new() {
        let old = r##"bind-key -T prefix X if-shell -F "#{@f0_path}" { run-shell -b "TMUX_PANE=#{pane_id} f0 done #{q:@f0_path}" } { display-message "Not an f0 session" }"##;
        assert!(is_f0_binding(old));
        assert!(is_f0_binding(&format!(
            "bind-key -T prefix X if-shell -F \"#{{@f0_path}}\" {}",
            done_command("/bin/f0")
        )));
        assert!(!is_f0_binding("bind-key -T prefix X kill-session"));
    }

    #[test]
    fn agent_command_asks_then_runs_this_f0_for_the_pressed_pane() {
        assert_eq!(
            agent_command("/a b/f0"),
            r##"command-prompt -p "Agent (Enter for the project's): " "run-shell -b \"TMUX_PANE=#{pane_id} '/a b/f0' agent -- %%% 2>&1\"""##
        );
    }

    #[test]
    fn each_key_knows_its_own_binding() {
        let bound = |kind: F0Key| {
            format!(
                "bind-key -T prefix A if-shell -F \"#{{@f0_path}}\" {}",
                kind.command("/bin/f0")
            )
        };
        let (agent, done) = (bound(F0Key::Agent), bound(F0Key::Done));
        assert!(is_f0_binding(&agent));
        assert!(F0Key::Agent.is_bound_in(&agent));
        assert!(!F0Key::Done.is_bound_in(&agent));
        assert!(F0Key::Done.is_bound_in(&done));
        assert!(!F0Key::Agent.is_bound_in(&done));
    }

    #[test]
    fn hints_name_the_prefix_and_each_key() {
        assert_eq!(
            key_hints("C-a", &[(F0Key::Done, "X")]),
            "C-a X close task  "
        );
        assert_eq!(
            key_hints("C-a", &[(F0Key::Done, "X"), (F0Key::Agent, "A")]),
            "C-a X close task  C-a A new agent  "
        );
    }

    #[test]
    fn agent_panes_are_marked_or_titled_and_sorted_top_down() {
        let out = "%1\t0\t40\t\tEDITOR\n\
                   %5\t31\t29\tcodex --full-auto\tAGENT: CODEX\n\
                   %2\t41\t18\t\tSHELL\n\
                   %3\t0\t30\tclaude\tAGENT: CLAUDE\n\
                   %9\t12\t5\t\tAGENT: OLD";
        let summary: Vec<_> = parse_agent_panes(out)
            .into_iter()
            .map(|p| (p.id, p.cmd, p.height))
            .collect();
        assert_eq!(
            summary,
            [
                ("%3".to_owned(), Some("claude".to_owned()), 30),
                ("%9".to_owned(), None, 5),
                ("%5".to_owned(), Some("codex --full-auto".to_owned()), 29)
            ]
        );
    }

    #[test]
    fn even_heights_fill_the_column_minus_borders() {
        assert_eq!(even_heights(40, 1), [40]);
        assert_eq!(even_heights(41, 2), [20, 20]);
        assert_eq!(even_heights(42, 2), [21, 20]);
        assert_eq!(even_heights(44, 3), [14, 14, 14]);
        assert_eq!(even_heights(46, 3), [15, 15, 14]);
        assert!(even_heights(10, 0).is_empty());
    }

    #[test]
    fn parses_size() {
        assert_eq!(parse_size("238 61\n"), Some((238, 61)));
        assert_eq!(parse_size(""), None);
        assert_eq!(parse_size("wide tall"), None);
    }

    #[test]
    fn parses_only_f0_sessions() {
        let out = "main\t\t1700000000\nhubi/HB-1\t/pool/1/hubi\t1700000123\nscratch\t\t1\nold/x\t/pool/2/old";
        assert_eq!(
            parse_sessions(out),
            [
                Session {
                    name: "hubi/HB-1".to_owned(),
                    path: PathBuf::from("/pool/1/hubi"),
                    activity: Some(1_700_000_123),
                },
                Session {
                    name: "old/x".to_owned(),
                    path: PathBuf::from("/pool/2/old"),
                    activity: None,
                },
            ]
        );
    }
}
