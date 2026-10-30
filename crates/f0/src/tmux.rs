use std::fs::File;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::error::F0Error;
use crate::layout::{Direction, Layout};
use crate::process;

const PATH_OPTION: &str = "@f0_path";
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
/// server, so it outlives every pane of the session.
pub fn confirm_then_run(
    session: &str,
    prompt: &str,
    template: &str,
    args: &[&str],
) -> Result<(), F0Error> {
    let client = tmux(&[
        "list-clients",
        "-t",
        &exact(session),
        "-F",
        "#{client_name}",
    ])?;
    let Some(client) = client.lines().next() else {
        return Err(F0Error::NoClient {
            session: session.to_owned(),
        });
    };
    let command = std::iter::once("run-shell -b".to_owned())
        .chain(
            std::iter::once(template)
                .chain(args.iter().copied())
                .map(quote),
        )
        .collect::<Vec<_>>()
        .join(" ");
    tmux(&[
        "confirm-before",
        "-t",
        client,
        "-p",
        &prompt.replace('#', "##"),
        &command,
    ])
    .map(drop)
}

/// Runs `template` with `run-shell -b` right away, like `confirm_then_run` once
/// confirmed.
pub fn run_in_background(template: &str, args: &[&str]) -> Result<(), F0Error> {
    let mut command = vec!["run-shell", "-b", template];
    command.extend(args);
    tmux(&command).map(drop)
}

/// `value` as one argument of a tmux command string, such as the command
/// `confirm-before` runs. Inside double quotes tmux expands `$VAR` and escapes.
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
