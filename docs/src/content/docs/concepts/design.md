---
title: Layers and decisions
description: Why f0 is split in layers and the choices behind it.
sidebar:
  order: 2
---

## Layers

| Layer | Needs | Commands |
|---|---|---|
| Worktree per task | treehouse, a project file and a `.worktreeinclude` | `f0 new`, `f0 open`, `f0 ls`, `f0 done` |
| Window layout | tmux | Built by `f0 new` and `f0 open` for every slot |
| Editor, tmux and terminal configs | nothing, installed on request | `install.sh` options |

The first two always go together. The configs are optional: f0 builds the same layout with your own editor and tmux setup.

## Decisions

**One command per task, one worktree per task.** Every workspace is a treehouse slot. A task never shares files, branches or servers with another one, and the agent can work in the background without touching your main checkout.

**Env files are part of a task.** A slot without its `.env` files cannot run its servers, so a project must list them in `.worktreeinclude`, even if the list is empty.

**Reuse treehouse instead of managing worktrees.** treehouse already handles the slot pool, leases, process cleanup and `.worktreeinclude`. Rebuilding that on top of `git worktree` would duplicate logic and bugs.

**tmux owns processes, the editor owns files.** Servers and the agent live in tmux panes, not editor terminals. They survive closing the editor or the terminal app, and every task can be detached and resumed.

**No state of its own.** f0 reads `treehouse status --json` and the tmux sessions every time, so nothing can drift.

**Leases, not subshells.** `treehouse get --lease` is non-interactive and keeps the slot reserved until it is returned, even when its session is detached.

**Commands are typed into shells.** Stopping a server with `Ctrl-C` leaves a shell to restart it.

**Servers wait for you.** A project lists what a task can run; you start what it needs with Enter, `--start` or `autostart`. Starting a whole monorepo for every task wastes memory and makes tasks fight over ports.

**Personal configuration only.** Project files live in your config folder, so shared repositories need no changes.

**Sessions are built at the final size.** Pane sizes are percentages of the window the session is created with, so f0 creates it at the size of your terminal instead of resizing it later.

**Never force a return.** treehouse asks before discarding uncommitted work, and f0 keeps that prompt.
