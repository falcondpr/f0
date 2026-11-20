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

## f0 launch

Show f0's home screen: your projects, then the tasks of the one you pick. [f0.app](/getting-started/installation/#f0app) runs it in every window, and it works in any other terminal too.

```bash
f0 launch
```

Projects and tasks show as a table under a search box, with the selected row in the accent color. A project row counts its tasks; a task row shows its branch, whether it is running (`●`) or stopped (`○`: the slot is leased, without a session), and when you last used it. The bottom of the table counts the rows and how many the search keeps. See [Key bindings](/reference/keybindings/#launcher) for every key.

- **Enter on a task** opens it like `f0 open` and attaches to its session. Leaving the session, by detaching (`Ctrl-a d`), with `f0 stop` or with `f0 done`, brings back the task list of that project, refreshed; Esc goes up to the projects.
- **`n`** asks for a ticket and starts the task like `f0 new`.
- **`a`**, on the projects, looks for git repositories that are not projects yet, up to three folders deep in the [search folders](/reference/configuration/#launcher), skipping `node_modules`, `target`, hidden folders and treehouse slots. Enter on one runs the questions of `f0 init` for it, and `p` takes the folder of one the search missed.
- **`q`** leaves the launcher, which closes the window of f0.app. The launcher has no way out to a plain shell.

Run inside tmux, it refuses and points to `f0 switch`: a launcher in a tmux pane would nest sessions.

## f0 init

Create the project file for a repository, asking about it.

```bash
f0 init [path] [--name <name>] [--template]
```

| Argument | Meaning |
|---|---|
| `[path]` | A folder inside the repository. Defaults to the current directory |
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
f0 new <ticket> [--project <name>] [--base <branch>] [--type <type>] [--start <servers>] [--agent <command>]... [--no-attach]
```

| Argument | Meaning |
|---|---|
| `<ticket>` | Task id, appended to the first of the project's `branch_prefixes` (`APP-123` becomes `feature/APP-123`) |
| `-p`, `--project` | Project name. Defaults to the project whose `repo` contains the current directory |
| `--base` | Branch to cut from, overriding `base` in the project |
| `-t`, `--type` | Branch type: one of the project's `branch_prefixes`, with or without the `/` (`--type fix` creates `fix/APP-123`). The session is still `my-app/APP-123` |
| `--start` | Servers to start right away, comma separated (`--start api,web`). The others wait for Enter, unless they have `autostart = true` |
| `--agent` | AI agent for this task only, like `codex` or `opencode`, overriding the project's. Repeat it to stack several agents in the agent column, the first on top: `--agent claude --agent codex` |
| `--no-attach` | Create the session without attaching, and print its name |

Requires treehouse, a project file and its [worktree include file](/guides/worktrees/#worktreeinclude). Fails before leasing a slot if any is missing, and if a session with the same name exists.

When the branch already exists, locally or on `origin`, as with a task returned before it was merged, the slot checks it out instead of creating it, and `--base` does nothing. It fails if another checkout has that branch: `f0 open` reopens it when it is a slot.

## f0 open

Reopen the workspace of a treehouse slot, attaching to its session if it is running.

```bash
f0 open [slot] [--start <servers>] [--agent <command>]... [--no-attach]
```

| Argument | Meaning |
|---|---|
| `[slot]` | Slot to open: its id (`my-app/4`), its task (`APP-123`), its session name or its path (any folder inside it works), as `f0 ls` shows them. Defaults to the current directory |
| `--start` | Servers to start when the session is rebuilt, as in `f0 new`. Does nothing if the session is already running |
| `--agent` | AI agents for the rebuilt session, as in `f0 new`, replacing the ones it had |
| `--no-attach` | Create the session without attaching |

A rebuilt session comes back with the agents it had when it was closed, including the ones added with [`f0 agent`](#f0-agent), and the first one resumes its last conversation in the task. The others start fresh: resuming by folder, like `claude --continue`, cannot tell their conversations apart. See [Resuming the conversation](/reference/configuration/#resuming-the-conversation).

Use it to get a task back after its session was closed, for example when tmux or the computer restarted. The slot must belong to a configured project; any other folder is refused.

An existing folder always wins over a name. A bare slot number (`4`) also works, but it repeats across projects: then the project of the current directory is used, and outside any project f0 lists the ids to pick from.

## f0 agent

Start another agent in the session of a task, under the ones it has.

```bash
f0 agent [command]
```

| Argument | Meaning |
|---|---|
| `[command]` | The agent and its arguments, like `codex` or `codex --full-auto`. Defaults to the project's agent |

Run it in any pane of a task's session, or press `Ctrl-a A`, which asks for the command in the status line (Enter alone starts the project's agent). The new pane goes under the last agent, in the slot's folder, titled `agent: codex`, or `agent: claude 2` for a second one of the same program; the agent column keeps its width and its agents share the height evenly. Outside an f0 session it fails without changing anything.

f0 remembers the agents of each slot, so `f0 stop` and `f0 open`, or the launcher, bring them back. An agent pane closed with `Ctrl-a x` stays closed. Returning the slot with `f0 done` forgets them, and the next task in that slot starts with the project's agent.

Every agent of a session works in the same folder and on the same branch. A second agent is for reviewing, testing or answering questions about the code while the first one works; two changes in parallel belong in two tasks.

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

Pick an f0 session from a table like that of [`f0 launch`](#f0-launch), most recently used first, and switch to it. Inside tmux it switches the current client; outside, it attaches. Enter switches, `/` searches, and Esc or `q` leaves without switching.

## f0 stop

Close a slot's session and keep the slot, with its branch and uncommitted changes, to come back later.

```bash
f0 stop [slot]
```

Takes the slot like `f0 done`, and defaults to the current directory. The agent and the servers stop; `f0 ls` keeps listing the slot, and `f0 open APP-123` rebuilds its session. Typed inside the session it closes, the session closes right away.

## f0 done

Close a slot's session and return the slot to the pool.

```bash
f0 done [slot] [--force]
```

Name the slot any way `f0 ls` shows it:

```bash
f0 done my-app/4                             # the ID column: unique across projects
f0 done APP-123                              # the TASK column
f0 done my-app/APP-123                       # the SESSION column
f0 done ~/.treehouse/my-app-3f9a2c/4/my-app  # the path, from `f0 ls --paths`
```

Without `[slot]` it uses the current directory. The session is closed first, then `treehouse return` cleans the slot, asking before it discards uncommitted changes; `--force` discards them without asking. Refuses folders that are not a slot.

Inside the session it would close, closing the session would stop `f0 done` itself halfway, so it asks in the tmux status line instead (`Close my-app/APP-123 and return its slot? (y/n)`, with the number of uncommitted changes it would discard, if any) and then your terminal leaves tmux and closes the session and returns the slot there, so you see the result. `Ctrl-a X` offers it from any pane, next to closing the session and keeping the slot (`f0 stop`), and the status line of the session shows the key. If returning the slot fails, the terminal shows why and waits for Enter, and the slot stays in `f0 ls`.

## f0 keys

Bind the f0 keys, the one that closes a task and the one that starts another agent, on the running tmux server and show them in the status line of every f0 session.

```bash
f0 keys
```

`f0 new` and `f0 open` do it for their sessions, and `install.sh` runs it after an update, so sessions that were already open get the new bindings without reloading tmux. The keys come from [`[keys]`](/reference/configuration/#keys); after changing one, run `f0 keys` to unbind the old one.

## f0 doctor

Check tools, global defaults, project configs, the tmux keys and what keeps the bundled theme from applying. Exits with an error when something required is missing.

| Check | Required |
|---|---|
| `tmux`, `git`, `treehouse` | Yes |
| At least one project | Yes |
| Editor and agent commands of the defaults and each project | Yes |
| Each project's `repo` and worktree include file | Yes |
| The running tmux server binds the keys that close a task and start another agent (skipped with no server, or for a key set to `false`) | Yes |
| `gh`, `rg`, Ghostty, JetBrainsMono Nerd Font | No |
| `/Applications/f0.app`, telling it from another app with that name, and its config `~/.config/f0/ghostty.conf` | No |
| With the bundled tmux config linked: tmux 3.8 or later, which the theme needs | Yes |
| With the bundled Ghostty config linked: no `background`, `foreground`, `palette`, `cursor-*` or `selection-*` lines in other Ghostty config files, listed with their file and line | No |

## Environment variables

| Variable | Read by | Meaning |
|---|---|---|
| `F0_CONFIG_DIR` | f0 | Config folder, overrides the default |
| `XDG_CONFIG_HOME` | f0 | Config folder becomes `$XDG_CONFIG_HOME/f0` |
| `F0_PROJECT` | panes | Set in every pane of a session: the project name |
| `F0_BASE` | panes | Set when the project has `base`; used by `Space gD` in Neovim |
