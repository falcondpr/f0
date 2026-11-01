---
title: Configuration
description: Global defaults and project files.
sidebar:
  order: 2
---

f0 reads only personal configuration, so shared repositories need no changes. You need at least one project; `config.toml` is optional.

Location, first match wins: `$F0_CONFIG_DIR`, `$XDG_CONFIG_HOME/f0`, `~/.config/f0`.

```
~/.config/f0/
├── config.toml          global defaults
└── projects/
    └── my-app.toml      one file per project
```

Unknown keys are rejected so typos fail loudly. Run `f0 doctor` after any change. `f0 init` asks a few questions and writes a project file for you, and `f0 edit` changes it the same way.

## Global defaults

`config.toml` sets what every workspace uses. Projects fall back to it for any section they leave out.

```toml
[defaults.editor]
cmd = "nvim ."

[defaults.agent]
cmd = "claude"

[defaults.layout]
terminals = 4
terminal_height = 25
agent_width = 35
```

| Key | Default | Meaning |
|---|---|---|
| `defaults.editor.cmd` | `nvim .` | Command for the editor pane |
| `defaults.agent.cmd` | `claude` | Command for the agent pane |
| `defaults.layout.terminals` | `4` | Panes in the bottom row, 1 to 6 |
| `defaults.layout.terminal_height` | `25` | Height of the bottom row, percent of the window, 10 to 90 |
| `defaults.layout.agent_width` | `35` | Width of the agent pane, percent of the top row, 10 to 90 |

Servers belong to projects and are not allowed here.

## Keys

f0 binds its tmux keys itself, after your tmux prefix, so they work with any tmux config and without reloading it. `f0 new` and `f0 open` bind them, and `f0 keys` does it for sessions that were already running.

```toml
[keys]
done = "X"
```

| Key | Default | Meaning |
|---|---|---|
| `keys.done` | `"X"` | Opens a menu to close the task of the session it is pressed in: keeping its slot, like `f0 stop`, or returning it, like `f0 done`. Any tmux key name (`"Q"`, `"M-x"`); `false` turns it off |

The status line of each f0 session shows the key, like `C-a X close task`. f0 never replaces a key your tmux config already binds to something else: `f0 keys` and `f0 doctor` tell you to pick another one.

## AI agent

The agent pane runs whatever command you give it, so any AI coding CLI that works in a terminal can be the agent: Claude Code (`claude`), Codex (`codex`), Gemini CLI (`gemini`), opencode (`opencode`) and so on. The pane is titled after the program, like `agent: codex`.

| To use it | Set |
|---|---|
| In every project | `[defaults.agent] cmd = "codex"` in `config.toml` |
| In one project | `[agent] cmd = "opencode"` in its file, or answer the question in `f0 init` and `f0 edit` |
| In one task | `f0 new APP-123 --agent gemini` |

Arguments are fine (`cmd = "codex --full-auto"`). `f0 new` and `f0 open` refuse to start when the agent's program is not installed, and `f0 doctor` checks every project's agent.

### Resuming the conversation

When `f0 open` rebuilds a closed session, for example after a restart, the agent picks up the task's last conversation. Every task has its own worktree, so it always resumes that task's conversation, not another one. Claude Code does it out of the box with `claude --continue`. For other agents, set the command that resumes, wherever you set `cmd`:

```toml
[agent]
cmd = "codex"
resume = "codex resume --last"
```

If the resume command fails, as when there is no conversation yet, the agent starts fresh. `f0 new` always starts a new conversation.

## Projects

`projects/<name>.toml`. The file name is the project name and the first part of every session name. f0 works only in repositories that have a project.

```toml
repo = "~/dev/my-app"
base = "develop"
branch_prefixes = ["feature/", "fix/", "refactor/", "chore/"]
# include_file = "~/.config/f0/my-app.worktreeinclude"

[[servers]]
name = "api"
cmd = "pnpm nx serve @org/api"

[[servers]]
name = "mobile"
cwd = "apps/mobile"
cmd = "npx expo start -c"

[agent]
cmd = "claude"

[layout]
terminals = 4
```

| Key | Default | Meaning |
|---|---|---|
| `repo` | required | Main checkout. `~/` is expanded. Matched against the current directory, also from inside a slot |
| `base` | inferred by treehouse | Branch new tasks are cut from. Exported as `F0_BASE` |
| `branch_prefixes` | `["feature/", "fix/", "refactor/", "chore/"]` | Prefixes for `f0 new` branches. The first is the default, `f0 new --type fix` picks `fix/`. Stripped from session names. Files that still have a single `branch_prefix = "feature/"` keep working with only that prefix |
| `include_file` | `<repo>/.worktreeinclude` | File that lists the ignored files copied into every slot. It must exist; an empty file copies nothing |
| `servers` | none | List of `name`, `cmd`, optional `cwd` (relative to the slot) and `autostart` (default `false`: the command waits for Enter) |
| `editor`, `agent`, `layout` | global defaults | Same keys as in `config.toml`, without the `defaults.` prefix |

The number of servers must fit in `layout.terminals`.

## Session names

| Where | Name |
|---|---|
| Task | `<project>/<task>`, for example `my-app/APP-123` |
| Slot with a detached HEAD | `<project>/slot-<n>` |

`.` and `:` become `_` because tmux uses them as separators.
