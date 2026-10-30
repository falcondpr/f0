# f0

A worktree-first terminal workspace for AI-assisted development, built on tmux, Neovim and treehouse.

f0 is a **Personal Development Environment (PDE)**: one repo holds the configs (Neovim, tmux, Ghostty) and a small Rust CLI that ties them to treehouse. Every task gets its own worktree slot and its own tmux session with the same layout:

```
+--------------------------------------------+------------------+
| nvim                                       | agent            |
|  changed files | file / diff | explorer    |                  |
+-----------+-----------+-----------+--------+------------------+
| server 1  | server 2  | server 3  | shell                     |
+-----------+-----------+-----------+---------------------------+
```

## Install

```bash
git clone <this repo> ~/dev/f0
~/dev/f0/install.sh
f0 doctor
```

`install.sh` installs tmux and treehouse with Homebrew (treehouse only if it is not already in `PATH`) and the `f0` binary with `cargo install`. By default it also installs Neovim, ripgrep, fd and Ghostty and links `config/nvim`, `config/tmux` and `config/ghostty/config` into `~/.config`. If you already have one of those configs it asks before replacing it, and keeps a backup. Run `install.sh --help` for the flags; `--cli-only` keeps all your own configs.

## Quickstart

One command per task: every task runs in its own treehouse worktree, so each repository needs a project file and a `.worktreeinclude` listing the env files to copy into every slot (it may be empty).

```bash
cd ~/dev/my-app
f0 init                         # asks about the repo and writes ~/.config/f0/projects/my-app.toml
f0 edit                         # change it later, question by question
f0 doctor
f0 new APP-123                  # lease a slot on feature/APP-123 and open the workspace
f0 new APP-124 --start api,web  # same, starting those servers; the rest wait for Enter
f0 new APP-125 --agent codex    # any AI CLI as the agent: claude, codex, gemini, opencode...
f0 ls                           # slots and sessions of every project
f0 switch                       # jump to another task
f0 open APP-123                 # rebuild a task's session, e.g. after a restart
f0 done APP-123                 # from outside the session: close it and return the slot
f0 done my-app/4                # same, by the slot id that `f0 ls` shows
```

## Docs

The documentation site lives in [`docs/`](docs/), in English and Spanish, built with Astro Starlight:

```bash
cd docs
pnpm install
pnpm dev
```

Pages are Markdown in `docs/src/content/docs/` (English) and `docs/src/content/docs/es/` (Spanish). Update both when behavior changes.
