//! `f0 edit`: walks through a project file question by question and rewrites it.

use std::fs;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::Path;
use std::process::Command;

use anyhow::{Context, bail};

use crate::config::{self, Agent, MAX_TERMINALS, Project, ProjectDraft, ProjectFile, Server};
use crate::error::F0Error;
use crate::git;
use crate::init::terminals_for;
use crate::prompt::{self, choose, confirm, prompt, split_list};

pub fn run(name: Option<&str>) -> anyhow::Result<()> {
    let config_dir = config::config_dir()?;
    let defaults = config::load_defaults(&config_dir)?;
    let projects = config::load_projects(&config_dir, &defaults)?;
    let project = match name {
        Some(name) => config::find_by_name(&projects, name, &config_dir)?,
        None => {
            let cwd = std::env::current_dir()?;
            let repo = git::main_repo_root(&cwd).map_err(|_| F0Error::NotARepo { path: cwd })?;
            config::find_by_repo(&projects, &repo, &config_dir)?
        }
    };
    let path = config_dir
        .join("projects")
        .join(format!("{}.toml", project.name));
    if !io::stdin().is_terminal() {
        return Err(F0Error::NeedsTerminal { path }.into());
    }

    let source = fs::read_to_string(&path).map_err(|source| F0Error::ReadConfig {
        path: path.clone(),
        source,
    })?;
    let file = ProjectFile::parse(&source, &path)?;
    let include = match &project.include_file {
        Some(include_path) => Some(read_include_file(include_path)?),
        None => None,
    };
    let detected_base = git::base_guess(&project.repo);

    println!("Editing {}. Press Enter to keep a value.", path.display());
    let answers = ask(
        &mut io::stdin().lock(),
        &mut io::stdout(),
        &file,
        include.as_deref(),
        detected_base.as_deref(),
        &defaults.agent.cmd,
    )?;

    // Keep `[agent]` out of the file while it matches the global default.
    let agent = (file.agent.is_some() || answers.agent != defaults.agent.cmd).then(|| Agent {
        cmd: answers.agent.clone(),
    });

    // Compare values, not text: a hand-written file renders differently even when every
    // answer kept its current value.
    let file_changed = answers.base != file.base
        || answers.branch_prefix != file.branch_prefix
        || answers.servers != file.servers
        || agent != file.agent;
    let include_changed = answers.include != include;
    if !file_changed && !include_changed {
        println!("No changes.");
        return Ok(());
    }

    let current_terminals = file
        .layout
        .as_ref()
        .map_or(defaults.layout.terminals, |l| l.terminals);
    let draft = ProjectDraft {
        repo: file.repo.clone(),
        base: answers.base,
        detected_base,
        branch_prefix: answers.branch_prefix,
        include_file: file.include_file.clone(),
        terminals: terminals_for(answers.servers.len(), current_terminals),
        servers: answers.servers,
        agent,
        editor: file.editor.clone(),
        layout: file.layout.clone(),
    };
    let contents = draft.render();
    let home = config::home_dir()?;
    Project::from_toml(&project.name, &contents, &path, &home, &defaults)?;

    // The new version sits next to the old one until you confirm, then replaces it in
    // one rename, so the project file is never half written.
    let staged = path.with_extension("toml.new");
    if file_changed {
        fs::write(&staged, &contents)?;
        show_diff(&path, &staged)?;
    }
    if include_changed
        && let (Some(patterns), Some(include_path)) = (&answers.include, &project.include_file)
    {
        println!(
            "Files copied into every slot ({}): {}",
            include_path.display(),
            if patterns.is_empty() {
                "none".to_owned()
            } else {
                patterns.join(", ")
            }
        );
    }

    let save = confirm(
        &mut io::stdin().lock(),
        &mut io::stdout(),
        "Save these changes?",
        true,
    );
    if !matches!(save, Ok(true)) && file_changed {
        let _ = fs::remove_file(&staged);
    }
    if !save? {
        println!("Nothing was saved.");
        return Ok(());
    }

    if file_changed {
        fs::rename(&staged, &path)?;
    }
    if include_changed
        && let (Some(patterns), Some(include_path)) = (&answers.include, &project.include_file)
    {
        let mut text = patterns.join("\n");
        if !text.is_empty() {
            text.push('\n');
        }
        fs::write(include_path, text)?;
    }
    println!("Saved. Run `f0 doctor` to check it.");
    Ok(())
}

fn read_include_file(path: &Path) -> anyhow::Result<Vec<String>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text =
        fs::read_to_string(path).with_context(|| format!("could not read {}", path.display()))?;
    Ok(text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect())
}

/// `git diff` exits with 1 when the files differ, which is the expected case here. It
/// runs next to the files so the header shows their names, not two long paths.
fn show_diff(old: &Path, new: &Path) -> anyhow::Result<()> {
    let name = |path: &Path| path.file_name().map(ToOwned::to_owned).unwrap_or_default();
    let mut command = Command::new("git");
    command
        .args(["diff", "--no-index", "--"])
        .arg(name(old))
        .arg(name(new));
    if let Some(dir) = old.parent() {
        command.current_dir(dir);
    }
    let status = command.status().context("could not run `git diff`")?;
    if !matches!(status.code(), Some(0 | 1)) {
        bail!("`git diff` failed: {status}");
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
struct Answers {
    base: Option<String>,
    branch_prefix: String,
    agent: String,
    /// `None` when the repo's own `.worktreeinclude` is used.
    include: Option<Vec<String>>,
    servers: Vec<Server>,
}

fn ask<R: BufRead, W: Write>(
    input: &mut R,
    out: &mut W,
    file: &ProjectFile,
    include: Option<&[String]>,
    detected_base: Option<&str>,
    default_agent: &str,
) -> anyhow::Result<Answers> {
    // A placeholder left by `f0 init --template` is no use as a default.
    let current_base = file
        .base
        .as_deref()
        .filter(|base| !config::is_placeholder(base))
        .or(detected_base);
    let base = prompt(input, out, "Base branch", current_base)?;
    let base = (!base.is_empty()).then_some(base);
    let branch_prefix = prompt(input, out, "Branch prefix", Some(&file.branch_prefix))?;
    let current_agent = file
        .agent
        .as_ref()
        .map_or(default_agent, |a| a.cmd.as_str());
    let agent = prompt::ask_agent(input, out, current_agent)?;

    let include = match include {
        None => {
            writeln!(out, "Env files: using the repo's .worktreeinclude")?;
            None
        }
        Some(patterns) => {
            if patterns.is_empty() {
                writeln!(out, "Files copied into every slot: none")?;
            } else {
                writeln!(out, "Files copied into every slot:")?;
                for pattern in patterns {
                    writeln!(out, "  {pattern}")?;
                }
            }
            if confirm(input, out, "Change them?", false)? {
                let answer = prompt(
                    input,
                    out,
                    "Files to copy, comma separated (empty for none)",
                    None,
                )?;
                Some(split_list(&answer))
            } else {
                Some(patterns.to_vec())
            }
        }
    };

    let mut servers: Vec<Server> = Vec::new();
    for (index, current) in file.servers.iter().enumerate() {
        writeln!(out, "Server {}", describe(current))?;
        let placeholder =
            config::is_placeholder(&current.name) || config::is_placeholder(&current.cmd);
        let choices: &[&str] = if placeholder {
            &["edit", "remove"]
        } else {
            &["keep", "edit", "remove"]
        };
        let question = if placeholder {
            "  It is a placeholder. Edit or remove?"
        } else {
            "  Keep, edit or remove?"
        };
        match choices[choose(input, out, question, choices)?] {
            "keep" => servers.push(current.clone()),
            "edit" => {
                let others: Vec<Server> = servers
                    .iter()
                    .chain(&file.servers[index + 1..])
                    .cloned()
                    .collect();
                servers.push(prompt::ask_server(input, out, Some(current), &others)?);
            }
            _ => {}
        }
    }

    let max = usize::from(MAX_TERMINALS);
    loop {
        if servers.len() == max {
            writeln!(out, "That is the most servers a window fits ({max}).")?;
            break;
        }
        let question = if servers.is_empty() {
            "Add a server?"
        } else {
            "Add another server?"
        };
        if !confirm(input, out, question, false)? {
            break;
        }
        servers.push(prompt::ask_server(input, out, None, &servers)?);
    }

    Ok(Answers {
        base,
        branch_prefix,
        agent,
        include,
        servers,
    })
}

fn describe(server: &Server) -> String {
    let mut text = format!("{}: {}", server.name, server.cmd);
    if let Some(cwd) = &server.cwd {
        text.push_str(&format!(" (in {})", cwd.display()));
    }
    if server.autostart {
        text.push_str(", starts with every task");
    }
    text
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::path::PathBuf;

    use super::*;

    fn file(source: &str) -> ProjectFile {
        ProjectFile::parse(source, Path::new("p.toml")).unwrap()
    }

    fn run_ask(
        answers: &str,
        file: &ProjectFile,
        include: Option<&[String]>,
    ) -> (anyhow::Result<Answers>, String) {
        let mut input = Cursor::new(answers.as_bytes().to_vec());
        let mut out = Vec::new();
        let result = ask(&mut input, &mut out, file, include, Some("main"), "claude");
        (result, String::from_utf8(out).unwrap())
    }

    const HUBI: &str = r#"
        repo = "~/dev/hubi"
        base = "develop"
        [[servers]]
        name = "nexus"
        cmd = "pnpm nx serve @org/nexus"
        [[servers]]
        name = "admin"
        cmd = "pnpm nx dev @org/admin"
        [[servers]]
        name = "mobile"
        cwd = "apps/mobile"
        cmd = "npx expo start -c"
        [agent]
        cmd = "claude"
    "#;

    #[test]
    fn enter_everywhere_keeps_the_file() {
        let file = file(HUBI);
        let (answers, out) = run_ask("\n\n\n\n\n\n\n", &file, None);
        let answers = answers.unwrap();
        assert_eq!(answers.base.as_deref(), Some("develop"));
        assert_eq!(answers.branch_prefix, "feature/");
        assert_eq!(answers.agent, "claude");
        assert_eq!(answers.include, None);
        assert_eq!(answers.servers, file.servers);
        assert!(
            out.contains("Server mobile: npx expo start -c (in apps/mobile)"),
            "{out}"
        );
    }

    #[test]
    fn edits_removes_and_adds_servers() {
        let file = file(HUBI);
        let answers = [
            "",
            "",
            "codex", // base, prefix, agent
            "e",
            "",
            "",
            "",
            "y", // nexus: keep values, autostart
            "r", // remove admin
            "",  // keep mobile
            "y",
            "mobile", // add: duplicate name
            "web",
            "pnpm dev:web",
            "",
            "",
            "n",
        ]
        .join("\n");
        let (answers, out) = run_ask(&format!("{answers}\n"), &file, None);
        let names: Vec<_> = answers
            .as_ref()
            .unwrap()
            .servers
            .iter()
            .map(|s| (s.name.as_str(), s.autostart))
            .collect();
        assert_eq!(names, [("nexus", true), ("mobile", false), ("web", false)]);
        assert!(out.contains("already a server called `mobile`"), "{out}");
        assert_eq!(answers.unwrap().agent, "codex");
    }

    #[test]
    fn placeholders_offer_the_detected_base_and_must_be_replaced() {
        let template = ProjectDraft::template(
            "/src/app".to_owned(),
            Some("/c/app.include".to_owned()),
            None,
        )
        .render();
        let file = file(&template);
        let answers = [
            "",
            "",
            "", // base takes `main`, prefix, agent
            "y",
            "apps/api/.env", // change env files
            "k",
            "e", // `keep` is not offered for a placeholder
            "web",
            "bun dev",
            "apps/web",
            "",
            "n",
        ]
        .join("\n");
        let include = vec![];
        let (answers, out) = run_ask(&format!("{answers}\n"), &file, Some(&include));
        let answers = answers.unwrap();
        assert_eq!(answers.base.as_deref(), Some("main"));
        assert_eq!(answers.include, Some(vec!["apps/api/.env".to_owned()]));
        assert_eq!(answers.servers[0].name, "web");
        assert_eq!(answers.servers[0].cwd, Some(PathBuf::from("apps/web")));
        assert!(out.contains("Base branch [main]"), "{out}");
        assert!(out.contains("Answer edit, remove"), "{out}");
    }
}
