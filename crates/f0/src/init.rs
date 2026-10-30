//! `f0 init`: asks about the repo you stand in and writes its project file.

use std::fs;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::Path;

use crate::config::{self, Agent, MAX_TERMINALS, Project, ProjectDraft, Server};
use crate::error::F0Error;
use crate::git;
use crate::prompt::{self, confirm, prompt};

/// Asks in a terminal; writes the placeholder template with `template` or when stdin is
/// not a terminal, so scripts never hang on a question.
pub fn run(name: Option<&str>, template: bool) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let repo = git::main_repo_root(&cwd).map_err(|_| F0Error::NotARepo { path: cwd })?;
    let repo = repo.canonicalize()?;
    let config_dir = config::config_dir()?;
    let defaults = config::load_defaults(&config_dir)?;
    let projects = config::load_projects(&config_dir, &defaults)?;
    let projects_dir = config_dir.join("projects");
    if let Some(existing) = config::project_for_repo(&projects, &repo) {
        return Err(F0Error::ProjectExists {
            path: projects_dir.join(format!("{}.toml", existing.name)),
            name: existing.name.clone(),
        }
        .into());
    }

    let home = config::home_dir()?;
    let default_name = match name {
        Some(name) => name.to_owned(),
        None => repo
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
    };
    let detected_base = git::base_guess(&repo);
    let has_manifest = repo.join(".worktreeinclude").exists();
    let check_name = |name: &str| -> Result<(), F0Error> {
        if !is_valid_project_name(name) {
            return Err(F0Error::InvalidProjectName {
                name: name.to_owned(),
            });
        }
        let path = projects_dir.join(format!("{name}.toml"));
        if path.exists() {
            return Err(F0Error::ProjectExists {
                name: name.to_owned(),
                path,
            });
        }
        Ok(())
    };

    let interactive = !template && io::stdin().is_terminal();
    let answers = if interactive {
        let env_files = if has_manifest {
            Vec::new()
        } else {
            eprintln!("Looking for env files...");
            git::ignored_env_files(&repo)?
        };
        let context = Context {
            default_name: default_name.clone(),
            detected_base: detected_base.as_deref(),
            has_manifest,
            env_files: &env_files,
            check_name: &check_name,
            default_agent: &defaults.agent.cmd,
        };
        Some(ask(&mut io::stdin().lock(), &mut io::stdout(), &context)?)
    } else {
        check_name(&default_name)?;
        None
    };
    let name = answers
        .as_ref()
        .map_or(default_name, |answers| answers.name.clone());

    let include_path = (!has_manifest).then(|| config_dir.join(format!("{name}.worktreeinclude")));
    let include_file = include_path
        .as_deref()
        .map(|path| config::with_tilde(path, &home));
    let repo_text = config::with_tilde(&repo, &home);
    let draft = match &answers {
        Some(answers) => ProjectDraft {
            repo: repo_text,
            base: answers.base.clone(),
            detected_base: detected_base.clone(),
            branch_prefix: answers.branch_prefix.clone(),
            include_file,
            servers: answers.servers.clone(),
            terminals: terminals_for(answers.servers.len(), defaults.layout.terminals),
            // The global default needs no copy in the project.
            agent: (answers.agent != defaults.agent.cmd).then(|| Agent {
                cmd: answers.agent.clone(),
            }),
            editor: None,
            layout: None,
        },
        None => ProjectDraft::template(repo_text, include_file, detected_base.clone()),
    };

    let file = projects_dir.join(format!("{name}.toml"));
    let contents = draft.render();
    // Whatever we render must load, or `f0 new` would fail later on a file we wrote.
    Project::from_toml(&name, &contents, &file, &home, &defaults)?;

    fs::create_dir_all(&projects_dir)?;
    if let Some(path) = &include_path {
        write_include_file(path, answers.as_ref().map_or(&[][..], |a| &a.env_patterns))?;
    }
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&file)?
        .write_all(contents.as_bytes())?;
    println!("Created {}", file.display());
    if answers.is_some() {
        println!("Run `f0 doctor`, then `f0 new <ticket>` to start a task.");
    } else {
        println!("Replace its <...> placeholders, then run `f0 doctor`.");
    }
    Ok(())
}

/// Keeps an include file that already exists: it may be shared with another checkout.
fn write_include_file(path: &Path, patterns: &[String]) -> io::Result<()> {
    if path.exists() {
        println!("Kept {}", path.display());
        return Ok(());
    }
    let mut contents = patterns.join("\n");
    if !contents.is_empty() {
        contents.push('\n');
    }
    fs::write(path, contents)?;
    match patterns.len() {
        0 => println!("Created {} (empty)", path.display()),
        n => println!("Created {} ({n} file(s))", path.display()),
    }
    Ok(())
}

/// Servers fill the terminal row first; keep a shell next to them when there is room.
pub fn terminals_for(servers: usize, default: u8) -> Option<u8> {
    let max = usize::from(MAX_TERMINALS);
    (servers > usize::from(default)).then(|| (servers + 1).min(max) as u8)
}

struct Context<'a> {
    default_name: String,
    detected_base: Option<&'a str>,
    has_manifest: bool,
    env_files: &'a [String],
    check_name: &'a dyn Fn(&str) -> Result<(), F0Error>,
    default_agent: &'a str,
}

#[derive(Debug, PartialEq)]
struct Answers {
    name: String,
    base: Option<String>,
    branch_prefix: String,
    agent: String,
    env_patterns: Vec<String>,
    servers: Vec<Server>,
}

fn ask<R: BufRead, W: Write>(
    input: &mut R,
    out: &mut W,
    context: &Context<'_>,
) -> anyhow::Result<Answers> {
    let name = loop {
        let name = prompt(input, out, "Project name", Some(&context.default_name))?;
        match (context.check_name)(&name) {
            Ok(()) => break name,
            Err(e) => writeln!(out, "  {e}")?,
        }
    };

    let base = prompt(input, out, "Base branch", context.detected_base)?;
    let base = (!base.is_empty()).then_some(base);
    let branch_prefix = prompt(input, out, "Branch prefix", Some("feature/"))?;
    let agent = prompt::ask_agent(input, out, context.default_agent)?;

    let mut env_patterns = Vec::new();
    if context.has_manifest {
        writeln!(out, "Env files: using the repo's .worktreeinclude")?;
    } else {
        if !context.env_files.is_empty() {
            writeln!(out, "Env files found, not tracked by git:")?;
            for file in context.env_files {
                writeln!(out, "  {file}")?;
            }
            if confirm(input, out, "Copy them into every slot?", true)? {
                env_patterns.extend(context.env_files.iter().cloned());
            }
        }
        let others = prompt(
            input,
            out,
            "Other files to copy, comma separated (empty for none)",
            None,
        )?;
        env_patterns.extend(prompt::split_list(&others));
    }

    let mut servers: Vec<Server> = Vec::new();
    let max = usize::from(MAX_TERMINALS);
    loop {
        if servers.len() == max {
            writeln!(out, "That is the most servers a window fits ({max}).")?;
            break;
        }
        let question = if servers.is_empty() {
            "Add a dev server?"
        } else {
            "Add another server?"
        };
        if !confirm(input, out, question, false)? {
            break;
        }
        servers.push(prompt::ask_server(input, out, None, &servers)?);
    }

    Ok(Answers {
        name,
        base,
        branch_prefix,
        agent,
        env_patterns,
        servers,
    })
}

fn is_valid_project_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('.')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn run_ask(answers: &str, context: &Context<'_>) -> (anyhow::Result<Answers>, String) {
        let mut input = io::Cursor::new(answers.as_bytes().to_vec());
        let mut out = Vec::new();
        let result = ask(&mut input, &mut out, context);
        (result, String::from_utf8(out).unwrap())
    }

    fn context<'a>(
        env_files: &'a [String],
        check_name: &'a dyn Fn(&str) -> Result<(), F0Error>,
    ) -> Context<'a> {
        Context {
            default_name: "bottom-sheet".to_owned(),
            detected_base: Some("main"),
            has_manifest: false,
            env_files,
            check_name,
            default_agent: "claude",
        }
    }

    fn any_name(name: &str) -> Result<(), F0Error> {
        if is_valid_project_name(name) {
            Ok(())
        } else {
            Err(F0Error::InvalidProjectName {
                name: name.to_owned(),
            })
        }
    }

    #[test]
    fn enter_everywhere_takes_the_defaults() {
        let env = vec!["apps/api/.env".to_owned()];
        let (answers, out) = run_ask("\n\n\n\n\n\n\n", &context(&env, &any_name));
        let answers = answers.unwrap();
        assert_eq!(answers.name, "bottom-sheet");
        assert_eq!(answers.base.as_deref(), Some("main"));
        assert_eq!(answers.branch_prefix, "feature/");
        assert_eq!(answers.agent, "claude");
        assert_eq!(answers.env_patterns, env);
        assert!(answers.servers.is_empty());
        assert!(out.contains("Base branch [main]: "), "{out}");
    }

    #[test]
    fn asks_again_until_the_answers_are_valid() {
        let answers = [
            "my app",         // invalid project name
            "sheet",          // project name
            "develop",        // base
            "fix/",           // prefix
            "codex",          // agent
            "n",              // skip the found env files
            "a/.env, b/.env", // other files
            "maybe",          // not y or n
            "y",              // add a server
            "",               // empty name
            "example",        // server name
            "",               // empty command
            "bun run example start",
            "example", // cwd
            "y",       // autostart
            "y",       // add another
            "example", // duplicate name
            "docs",
            "bun docs",
            "",
            "",  // autostart: default no
            "n", // no more servers
        ]
        .join("\n");
        let env = vec!["apps/api/.env".to_owned()];
        let (result, out) = run_ask(&format!("{answers}\n"), &context(&env, &any_name));
        let answers = result.unwrap();
        assert_eq!(answers.name, "sheet");
        assert_eq!(answers.base.as_deref(), Some("develop"));
        assert_eq!(answers.branch_prefix, "fix/");
        assert_eq!(answers.agent, "codex");
        assert_eq!(answers.env_patterns, ["a/.env", "b/.env"]);
        assert_eq!(
            answers.servers,
            [
                Server {
                    name: "example".to_owned(),
                    cmd: "bun run example start".to_owned(),
                    cwd: Some(PathBuf::from("example")),
                    autostart: true,
                },
                Server {
                    name: "docs".to_owned(),
                    cmd: "bun docs".to_owned(),
                    cwd: None,
                    autostart: false,
                },
            ]
        );
        assert!(out.contains("cannot be a project name"), "{out}");
        assert!(out.contains("Answer y or n"), "{out}");
        assert!(out.contains("without spaces or commas"), "{out}");
        assert!(out.contains("The command is required"), "{out}");
        assert!(out.contains("already a server called `example`"), "{out}");
    }

    #[test]
    fn repo_manifest_skips_the_env_questions() {
        let mut context = context(&[], &any_name);
        context.has_manifest = true;
        context.detected_base = None;
        let (answers, out) = run_ask("\n\n\n\nn\n", &context);
        let answers = answers.unwrap();
        assert_eq!(answers.base, None);
        assert!(answers.env_patterns.is_empty());
        assert!(out.contains("using the repo's .worktreeinclude"), "{out}");
        assert!(!out.contains("Other files"), "{out}");
    }

    #[test]
    fn end_of_input_cancels() {
        let (result, _) = run_ask("sheet\n", &context(&[], &any_name));
        let err = result.unwrap_err().to_string();
        assert!(err.contains("nothing was written"), "{err}");
    }

    #[test]
    fn terminal_row_grows_only_when_servers_need_it() {
        assert_eq!(terminals_for(3, 4), None);
        assert_eq!(terminals_for(4, 4), None);
        assert_eq!(terminals_for(5, 4), Some(6));
        assert_eq!(terminals_for(6, 4), Some(6));
    }

    #[test]
    fn project_names_are_plain_file_names() {
        assert!(is_valid_project_name("bottom-sheet"));
        assert!(is_valid_project_name("my_app.v2"));
        assert!(!is_valid_project_name(""));
        assert!(!is_valid_project_name(".hidden"));
        assert!(!is_valid_project_name("a/b"));
        assert!(!is_valid_project_name("my app"));
    }
}
