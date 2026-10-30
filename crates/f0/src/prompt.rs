//! Questions in the terminal, shared by `f0 init` and `f0 edit`. Generic over the input
//! and output so tests can answer them.

use std::io::{BufRead, Write};
use std::path::PathBuf;

use anyhow::bail;

use crate::config::{self, Agent, Server};
use crate::process;

/// Reads one answer; an empty one takes `default`. End of input cancels before anything
/// is written.
pub fn prompt<R: BufRead, W: Write>(
    input: &mut R,
    out: &mut W,
    question: &str,
    default: Option<&str>,
) -> anyhow::Result<String> {
    match default.filter(|d| !d.is_empty()) {
        Some(default) => write!(out, "{question} [{default}]: ")?,
        None => write!(out, "{question}: ")?,
    }
    out.flush()?;
    let mut line = String::new();
    if input.read_line(&mut line)? == 0 {
        writeln!(out)?;
        bail!("cancelled, nothing was written");
    }
    let answer = line.trim();
    Ok(if answer.is_empty() {
        default.unwrap_or_default().to_owned()
    } else {
        answer.to_owned()
    })
}

pub fn confirm<R: BufRead, W: Write>(
    input: &mut R,
    out: &mut W,
    question: &str,
    default: bool,
) -> anyhow::Result<bool> {
    let hint = if default { "Y/n" } else { "y/N" };
    loop {
        let answer = prompt(input, out, &format!("{question} [{hint}]"), None)?;
        match answer.to_ascii_lowercase().as_str() {
            "" => return Ok(default),
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => writeln!(out, "  Answer y or n")?,
        }
    }
}

/// The agent command. Any CLI that runs in a terminal works; one that is not installed
/// yet is accepted with a warning, since `f0 doctor` keeps flagging it.
pub fn ask_agent<R: BufRead, W: Write>(
    input: &mut R,
    out: &mut W,
    current: &str,
) -> anyhow::Result<String> {
    loop {
        let cmd = prompt(
            input,
            out,
            "AI agent command (claude, codex, gemini, opencode...)",
            Some(current),
        )?;
        if cmd.is_empty() {
            writeln!(out, "  The agent command is required, like `claude`")?;
            continue;
        }
        let program = Agent::new(cmd.clone());
        if !process::is_installed(program.program()) {
            writeln!(
                out,
                "  `{}` is not installed yet; `f0 doctor` will flag it until it is",
                program.program()
            )?;
        }
        return Ok(cmd);
    }
}

/// Branch prefixes, comma separated; the first is what `f0 new` uses without `--type`.
pub fn ask_branch_prefixes<R: BufRead, W: Write>(
    input: &mut R,
    out: &mut W,
    current: &[String],
) -> anyhow::Result<Vec<String>> {
    let current = current.join(", ");
    loop {
        let answer = prompt(
            input,
            out,
            "Branch prefixes, comma separated, the first is the default",
            Some(&current),
        )?;
        let prefixes = split_list(&answer);
        match config::check_branch_prefixes(&prefixes) {
            Ok(()) => return Ok(prefixes),
            Err(reason) => writeln!(out, "  {reason}")?,
        }
    }
}

/// A comma separated answer as a list, without empty entries.
pub fn split_list(answer: &str) -> Vec<String> {
    answer
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_owned)
        .collect()
}

/// One of `choices` by its first letter; the first choice is the default.
pub fn choose<R: BufRead, W: Write>(
    input: &mut R,
    out: &mut W,
    question: &str,
    choices: &[&str],
) -> anyhow::Result<usize> {
    let hint = choices
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let letter = &c[..1];
            if i == 0 {
                letter.to_ascii_uppercase()
            } else {
                letter.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("/");
    loop {
        let answer = prompt(input, out, &format!("{question} [{hint}]"), None)?;
        let answer = answer.to_ascii_lowercase();
        if answer.is_empty() {
            return Ok(0);
        }
        if let Some(index) = choices
            .iter()
            .position(|c| answer == *c || answer == c[..1])
        {
            return Ok(index);
        }
        writeln!(out, "  Answer {}", choices.join(", "))?;
    }
}

/// Asks for a server, offering `current` as the default answers. `others` are the
/// project's other servers, whose names it must not repeat.
pub fn ask_server<R: BufRead, W: Write>(
    input: &mut R,
    out: &mut W,
    current: Option<&Server>,
    others: &[Server],
) -> anyhow::Result<Server> {
    let name = loop {
        let name = prompt(input, out, "  Name", current.map(|s| s.name.as_str()))?;
        if name.is_empty() || name.contains(|c: char| c == ',' || c.is_whitespace()) {
            writeln!(out, "  Use a name without spaces or commas, like `web`")?;
        } else if others.iter().any(|s| s.name == name) {
            writeln!(out, "  There is already a server called `{name}`")?;
        } else {
            break name;
        }
    };
    let cmd = loop {
        let cmd = prompt(input, out, "  Command", current.map(|s| s.cmd.as_str()))?;
        if cmd.is_empty() {
            writeln!(out, "  The command is required, like `pnpm dev`")?;
        } else {
            break cmd;
        }
    };
    let current_cwd = current
        .and_then(|s| s.cwd.as_ref())
        .map(|cwd| cwd.to_string_lossy().into_owned());
    let cwd = prompt(
        input,
        out,
        "  Folder, relative to the repo",
        Some(current_cwd.as_deref().unwrap_or(".")),
    )?;
    let autostart = confirm(
        input,
        out,
        "  Start it with every task?",
        current.is_some_and(|s| s.autostart),
    )?;
    Ok(Server {
        name,
        cmd,
        cwd: (cwd != "." && !cwd.is_empty()).then(|| PathBuf::from(cwd)),
        autostart,
    })
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    fn answer<T>(
        text: &str,
        f: impl FnOnce(&mut Cursor<Vec<u8>>, &mut Vec<u8>) -> anyhow::Result<T>,
    ) -> (anyhow::Result<T>, String) {
        let mut input = Cursor::new(text.as_bytes().to_vec());
        let mut out = Vec::new();
        let result = f(&mut input, &mut out);
        (result, String::from_utf8(out).unwrap())
    }

    #[test]
    fn editing_a_server_keeps_what_you_skip() {
        let current = Server {
            name: "mobile".to_owned(),
            cmd: "npx expo start -c".to_owned(),
            cwd: Some(PathBuf::from("apps/mobile")),
            autostart: true,
        };
        let (server, out) = answer("\nnpx expo start\n\n\n", |i, o| {
            ask_server(i, o, Some(&current), &[])
        });
        let server = server.unwrap();
        assert_eq!(server.cmd, "npx expo start");
        assert_eq!(server.cwd, current.cwd);
        assert!(server.autostart);
        assert!(
            out.contains("Folder, relative to the repo [apps/mobile]"),
            "{out}"
        );

        let (server, _) = answer("\n\n.\nn\n", |i, o| ask_server(i, o, Some(&current), &[]));
        let server = server.unwrap();
        assert_eq!(server.cwd, None);
        assert!(!server.autostart);
    }

    #[test]
    fn choose_takes_letters_words_or_the_default() {
        let choices = ["keep", "edit", "remove"];
        for (text, expected) in [("\n", 0), ("e\n", 1), ("remove\n", 2), ("x\nr\n", 2)] {
            let (index, _) = answer(text, |i, o| choose(i, o, "Keep?", &choices));
            assert_eq!(index.unwrap(), expected, "{text:?}");
        }
        let (_, out) = answer("\n", |i, o| choose(i, o, "Keep, edit or remove?", &choices));
        assert!(out.contains("[K/e/r]"), "{out}");
    }
}
