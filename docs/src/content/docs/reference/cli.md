---
title: CLI
description: Every f0 command and flag.
sidebar:
  order: 1
---

```
f0 <command> [options]
```

Run `f0 <command> --help` for the same information in the terminal.

## f0 init

Create the project file for the repository you are in, asking about it.

```bash
f0 init [--name <name>] [--template]
```

| Argument | Meaning |
|---|---|
| `--name` | Default answer for the project name. Otherwise the repository folder name |
| `--template` | Write a file with `<...>` placeholders instead of asking |

It asks for the project name, base branch (default: where `origin/HEAD` points, or the current branch in a repo without a remote), branch prefixes, [AI agent](/reference/configuration/#ai-agent), env files and dev servers. Press Enter to take the value in brackets.

```
Project name [react-native-bottom-sheet]: bottom-sheet
Base branch [main]:
Branch prefixes, comma separated, the first is the default [feature/, fix/, refactor/, chore/]:
AI agent command (claude, codex, gemini, opencode...) [claude]:
Other files to copy, comma separated (empty for none):
Add a dev server? [y/N]: y
  Name: example
  Command: bun run example start
  Folder, relative to the repo (empty for the root):
  Start it with every task? [y/N]:
Add another server? [y/N]:
```

When the repository has no `.worktreeinclude`, f0 lists the ignored `.env*` files it finds and writes the ones you pick to `<name>.worktreeinclude` in the config folder, pointing `include_file` at it. With its own `.worktreeinclude`, the repository keeps using it and f0 skips the question. With more servers than terminals, the file also raises `layout.terminals`.

Without a terminal, as in a script, or with `--template`, it writes the placeholder version instead; `f0 new`, `f0 open` and `f0 doctor` reject a project while a placeholder is left. `Ctrl-D` cancels before anything is written, and it never overwrites: it fails if the repository already has a project or the file exists.

## f0 edit

Update a project file question by question, showing the changes before saving.

```bash
f0 edit [project]
```

| Argument | Meaning |
|---|---|
| `[project]` | Project to edit. Defaults to the project of the repository you are in |

It asks the same questions as `f0 init`, with the current values as defaults, so Enter keeps them. For each server you choose to keep, edit or remove it, then you can add more.

```
Base branch [develop]:
Branch prefixes, comma separated, the first is the default [feature/, fix/, refactor/, chore/]:
AI agent command (claude, codex, gemini, opencode...) [claude]:
Env files: using the repo's .worktreeinclude
Server api: pnpm nx serve @org/api
  Keep, edit or remove? [K/e/r]: e
  Name [api]:
  Command [pnpm nx serve @org/api]:
  Folder, relative to the repo [.]:
  Start it with every task? [y/N]: y
Server admin: pnpm nx dev @org/admin
  Keep, edit or remove? [K/e/r]: r
Add another server? [y/N]:
```

If nothing changed it says so and leaves the file alone. Otherwise it shows a `git diff` of the file and asks before saving; the new file replaces the old one in a single step. `editor`, `agent` and `layout` are kept, comments you wrote yourself are not. Placeholders from `f0 init --template` are offered as questions to fill in. It needs a terminal.

## f0 new

Lease a treehouse slot with a new branch and open its workspace.

```bash
f0 new <ticket> [--project <name>] [--base <branch>] [--type <type>] [--start <servers>] [--agent <command>] [--no-attach]
```

| Argument | Meaning |
|---|---|
| `<ticket>` | Task id, appended to the first of the project's `branch_prefixes` (`APP-123` becomes `feature/APP-123`) |
| `-p`, `--project` | Project name. Defaults to the project whose `repo` contains the current directory |
| `--base` | Branch to cut from, overriding `base` in the project |
| `-t`, `--type` | Branch type: one of the project's `branch_prefixes`, with or without the `/` (`--type fix` creates `fix/APP-123`). The session is still `my-app/APP-123` |
| `--start` | Servers to start right away, comma separated (`--start api,web`). The others wait for Enter, unless they have `autostart = true` |
| `--agent` | AI agent for this task only, like `codex` or `opencode`, overriding the project's |
| `--no-attach` | Create the session without attaching, and print its name |

Requires treehouse, a project file and its [worktree include file](/guides/worktrees/#worktreeinclude). Fails before leasing a slot if any is missing, and if a session with the same name exists.

## f0 open

Reopen the workspace of a treehouse slot, attaching to its session if it is running.

```bash
f0 open [slot] [--start <servers>] [--agent <command>] [--no-attach]
```

| Argument | Meaning |
|---|---|
| `[slot]` | Slot to open: its id (`my-app/4`), its task (`APP-123`), its session name or its path (any folder inside it works), as `f0 ls` shows them. Defaults to the current directory |
| `--start` | Servers to start when the session is rebuilt, as in `f0 new`. Does nothing if the session is already running |
| `--agent` | AI agent for the rebuilt session, as in `f0 new` |
| `--no-attach` | Create the session without attaching |

A rebuilt session resumes the agent's last conversation in the task. See [Resuming the conversation](/reference/configuration/#resuming-the-conversation).

Use it to get a task back after its session was closed, for example when tmux or the computer restarted. The slot must belong to a configured project; any other folder is refused.

An existing folder always wins over a name. A bare slot number (`4`) also works, but it repeats across projects: then the project of the current directory is used, and outside any project f0 lists the ids to pick from.

## f0 ls

List the slots in use, with their task and session.

```bash
f0 ls [--all] [--paths]
```

```
ID         TASK        STATUS  SESSION
website/1  (detached)  in-use  website/slot-1
my-app/4   APP-123     leased  my-app/APP-123
Free slots: my-app 2 (`f0 ls --all` lists them)
```

| Argument | Meaning |
|---|---|
| `--all` | Also list free slots |
| `--paths` | Add a column with the folder of each slot |

treehouse numbers slots per repository, so the same number shows up in several projects. The **ID** column, `<project>/<slot>`, is unique, and `f0 open` and `f0 done` accept it. **TASK** is the branch without whichever of the project's `branch_prefixes` it starts with.

## f0 switch

Show a numbered list of f0 sessions and switch to the one you pick. Inside tmux it switches the current client; outside, it attaches.

## f0 done

Close a slot's session and return the slot to the pool.

```bash
f0 done [slot]
```

Name the slot any way `f0 ls` shows it:

```bash
f0 done my-app/4                             # the ID column: unique across projects
f0 done APP-123                              # the TASK column
f0 done my-app/APP-123                       # the SESSION column
f0 done ~/.treehouse/my-app-3f9a2c/4/my-app  # the path, from `f0 ls --paths`
```

Without `[slot]` it uses the current directory. The session is closed first, then `treehouse return` cleans the slot, asking before it discards uncommitted changes. Refuses folders that are not a slot, and refuses to run from inside the session it would close.

## f0 doctor

Check tools, global defaults and project configs. Exits with an error when something required is missing.

| Check | Required |
|---|---|
| `tmux`, `git`, `treehouse` | Yes |
| At least one project | Yes |
| Editor and agent commands of the defaults and each project | Yes |
| Each project's `repo` and worktree include file | Yes |
| `gh`, `rg`, Ghostty | No |

## Environment variables

| Variable | Read by | Meaning |
|---|---|---|
| `F0_CONFIG_DIR` | f0 | Config folder, overrides the default |
| `XDG_CONFIG_HOME` | f0 | Config folder becomes `$XDG_CONFIG_HOME/f0` |
| `F0_PROJECT` | panes | Set in every pane of a session: the project name |
| `F0_BASE` | panes | Set when the project has `base`; used by `Space gD` in Neovim |
