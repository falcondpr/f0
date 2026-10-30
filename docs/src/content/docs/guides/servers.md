---
title: Servers
description: Run, restart and organize dev servers in the terminal row.
sidebar:
  order: 3
---

Each `[[servers]]` entry in a project file gets its own pane in the bottom row, in order. Remaining panes up to `layout.terminals` are plain shells.

```toml
[[servers]]
name = "api"
cmd = "pnpm nx serve @org/api"

[[servers]]
name = "mobile"
cwd = "apps/mobile"
cmd = "npx expo start -c"
```

- `name` is shown in the pane border.
- `cwd` is relative to the slot.
- The command is typed into a shell, not run as the pane's process. `Ctrl-C` stops the server and leaves the shell open, so `Up` then `Enter` restarts it.

## Choosing what starts

The list is what a task *can* run, not what it runs. By default no server starts: each pane opens with its command typed at the prompt, waiting for Enter. Most tasks touch one or two apps of a monorepo, so starting everything would waste memory and fight over ports.

| To | Do |
|---|---|
| Start a server now | Press Enter in its pane |
| Start some servers with the task | `f0 new APP-123 --start api,web` |
| Start a server with every task | `autostart = true` in its `[[servers]]` block |
| Start servers when rebuilding a session | `f0 open APP-123 --start api` |

```toml
[[servers]]
name = "api"
cmd = "pnpm nx serve @org/api"
autostart = true
```

`--start` checks the names before leasing a slot, so a typo fails early and lists the project's servers.

## Things to know

- A server runs the code of the folder it started in. Servers of another task keep serving that task's code.
- Two tasks running the same server will fight over the same port. Stop one, or configure different ports per task.
- Zoom a pane with `Ctrl-a z` to read long logs, and scroll back with `Ctrl-a [`.
- `layout.terminals` must be at least the number of servers (maximum 6).
