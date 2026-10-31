use std::fs::File;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::error::F0Error;
use crate::layout::{Direction, Layout};
use crate::process;

const PATH_OPTION: &str = "@f0_path";
/// The key f0 last bound to close a task, kept on the tmux server so a changed or
/// turned off `[keys] done` can unbind it.
const DONE_KEY_OPTION: &str = "@f0_done_key";
const DEFAULT_SIZE: (u16, u16) = (240, 60);

#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    pub name: String,
    pub path: PathBuf,
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
    let format = format!("#{{session_name}}\t#{{{PATH_OPTION}}}");
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

fn client_of(session: &str) -> Result<String, F0Error> {
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

/// The key tmux waits for before a bound key, like `C-a`.
pub fn prefix() -> Result<String, F0Error> {
    tmux(&["show-options", "-gv", "prefix"])
}

/// Binds `key`, after the prefix, to `f0 done` for the session it is pressed in, and
/// unbinds the key an earlier f0 bound when it is a different one. `None` only unbinds.
/// It never replaces a key bound to something else.
pub fn bind_done(key: Option<&str>, exe: &str) -> Result<(), F0Error> {
    let previous = tmux(&["show-options", "-gqv", DONE_KEY_OPTION])?;
    if !previous.is_empty()
        && Some(previous.as_str()) != key
        && prefix_binding(&previous)?.is_some_and(|binding| is_f0_binding(&binding))
    {
        tmux(&["unbind-key", "-T", "prefix", &previous])?;
    }
    let Some(key) = key else {
        return tmux(&["set-option", "-gu", DONE_KEY_OPTION]).map(drop);
    };
    if let Some(binding) = prefix_binding(key)?
        && !is_f0_binding(&binding)
    {
        return Err(F0Error::KeyTaken {
            key: key.to_owned(),
            binding,
        });
    }
    let run = done_command(exe);
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
                key: key.to_owned(),
            }
        }
        e => e,
    })?;
    tmux(&["set-option", "-g", DONE_KEY_OPTION, key]).map(drop)
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

/// Binding from `bind_done`, or from the `bind X` block older f0 tmux configs had, which
/// ran `f0 done`.
pub fn is_f0_binding(binding: &str) -> bool {
    binding.contains(PATH_OPTION)
        && (binding.contains(" done ") || binding.contains(" close-menu "))
}

/// Puts `key` at the start of the status line of `session`, before the global
/// `status-right`, which gets that much more room so its end is not cut. Built from the
/// global values every time, so applying it again does not repeat it. `None` brings back
/// the global status line.
pub fn show_done_hint(session: &str, key: Option<&str>) -> Result<(), F0Error> {
    let target = format!("{}:", exact(session));
    let Some(key) = key else {
        tmux(&["set-option", "-u", "-t", &target, "status-right"])?;
        return tmux(&["set-option", "-u", "-t", &target, "status-right-length"]).map(drop);
    };
    let status = process::line_output("tmux", ["show-options", "-gv", "status-right"], None)?;
    let length: usize = tmux(&["show-options", "-gv", "status-right-length"])?
        .parse()
        .unwrap_or_default();
    let hint = done_hint(&prefix()?, key);
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

fn done_hint(prefix: &str, key: &str) -> String {
    format!("{prefix} {key} close task  ")
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
        // Agents like Claude Code retitle their pane with escape sequences; the title
        // f0 sets is what tells the panes apart. tmux before 3.4 lacks the option.
        let _ = tmux(&["set-option", "-p", "-t", id, "allow-set-title", "off"]);
        tmux(&["select-pane", "-t", id, "-T", &pane.title])?;
        if let Some(command) = &pane.command {
            tmux(&["send-keys", "-t", id, "-l", command])?;
            if pane.run {
                tmux(&["send-keys", "-t", id, "Enter"])?;
            }
        }
    }

    tmux(&["select-pane", "-t", &ids[layout.focus]])?;
    Ok(())
}

pub fn kill_session(name: &str) -> Result<(), F0Error> {
    tmux(&["kill-session", "-t", &exact(name)]).map(drop)
}

/// Moves the current client to `name`, or replaces this process with `tmux attach`
/// when running outside tmux.
pub fn attach(name: &str) -> Result<(), F0Error> {
    let target = exact(name);
    if std::env::var_os("TMUX").is_some() {
        return tmux(&["switch-client", "-t", &target]).map(drop);
    }
    let source = Command::new("tmux")
        .args(["attach-session", "-t", &target])
        .exec();
    Err(F0Error::Spawn {
        program: "tmux".to_owned(),
        source,
    })
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
            let (name, path) = line.split_once('\t')?;
            (!path.is_empty()).then(|| Session {
                name: name.to_owned(),
                path: PathBuf::from(path),
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
    fn hint_names_the_prefix_and_the_key() {
        assert_eq!(done_hint("C-a", "X"), "C-a X close task  ");
    }

    #[test]
    fn parses_size() {
        assert_eq!(parse_size("238 61\n"), Some((238, 61)));
        assert_eq!(parse_size(""), None);
        assert_eq!(parse_size("wide tall"), None);
    }

    #[test]
    fn parses_only_f0_sessions() {
        let out = "main\t\nhubi/HB-1\t/pool/1/hubi\nscratch\t";
        assert_eq!(
            parse_sessions(out),
            [Session {
                name: "hubi/HB-1".to_owned(),
                path: PathBuf::from("/pool/1/hubi"),
            }]
        );
    }
}
