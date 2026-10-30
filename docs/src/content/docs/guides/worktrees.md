---
title: Worktrees
description: How f0 uses treehouse slots and .worktreeinclude.
sidebar:
  order: 2
---

AI agents work best with one checkout per task, so two tasks never touch the same files or servers. f0 delegates this to treehouse, which keeps a pool of reusable git worktrees called **slots**.

## Lifecycle of a slot

| Step | Command | What happens |
|---|---|---|
| Lease | `f0 new APP-123` | treehouse hands out a free slot (or creates one), checks out a new branch from the base and seeds ignored files |
| Work | `f0 open`, `f0 switch` | The slot stays leased while you work, even with the session detached |
| Return | `f0 done APP-123` or `f0 done my-app/4` | Leftover processes are stopped, the slot is cleaned and goes back to the pool |

Slots live at `~/.treehouse/<repo>-<id>/<slot>/<repo>`. `f0 ls` shows the slots in use as `<project>/<slot>`, an id that is unique across projects, with their task and session.

## .worktreeinclude

Every project needs one: f0 does not start a task without it. A new slot only has tracked files. Ignored files such as `.env` are copied from the main checkout if they match a pattern in `.worktreeinclude` at the repository root, one per line:

```
apps/api/.env
apps/web/.env
apps/mobile/.env.development
```

An empty file is valid and copies nothing. The copy happens when the slot is leased. After changing a `.env` in the main checkout, copy it into running slots yourself or start a new task.

### Without committing the file

If the repository is shared and you cannot commit `.worktreeinclude`, keep it in your config folder and point the project at it:

```toml
include_file = "~/.config/f0/my-app.worktreeinclude"
```

f0 passes it as `treehouse get --include-file`.

## Running `done` inside its own session

Returning a slot stops every process inside it, including the shell that would be running `f0 done`. So inside the task's session, `f0 done` (or `Ctrl-a X`) asks in the tmux status line and leaves the work to a tmux background job, which no pane owns. That job cannot answer treehouse's question about uncommitted changes, so f0 asks first and tells you how many would be discarded.
