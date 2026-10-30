---
title: Architecture
description: How the f0 CLI, tmux, Neovim and treehouse fit together.
sidebar:
  order: 1
---

f0 is a **Personal Development Environment (PDE)**: configs plus a small Rust CLI that ties them together. The CLI does not keep state of its own. It reads treehouse and tmux, and drives them.

```
f0 (Rust CLI)
 ├── treehouse  lease, list and return worktree slots
 ├── tmux       one session per workspace, built pane by pane
 └── git        find the checkout, the main repository and the branch
config/
 ├── nvim       editor, explorer, diffs
 ├── tmux       prefix, navigation, pane titles
 └── ghostty    font and keyboard
```

## What `f0 new` does

1. Loads `config.toml` and `projects/*.toml`, and picks the project whose `repo` is the main checkout of the current directory.
2. Runs `treehouse get --lease --lease-holder f0:<session> --branch <prefix><ticket>`, adding `--base` and `--include-file` when configured. treehouse prints the slot path.
3. Turns the project into a list of panes and splits. This step is pure and unit tested.
4. Creates the tmux session at the size of your terminal, splits it pane by pane, names each pane and types each command into it.
5. Stores the slot path on the session as the `@f0_path` option. `ls`, `open` and `done` find sessions through it.
6. Attaches, or switches the client when already inside tmux.

## Source layout

| Path | Contents |
|---|---|
| `crates/f0/src/main.rs` | Command line definition |
| `crates/f0/src/commands.rs` | One function per command |
| `crates/f0/src/config.rs` | Config files, defaults and validation |
| `crates/f0/src/layout.rs` | Pure description of the window |
| `crates/f0/src/tmux.rs` | Session creation and lookup |
| `crates/f0/src/treehouse.rs` | treehouse calls |
| `config/` | Bundled Neovim, tmux and Ghostty configs |
| `install.sh` | Installer |
| `docs/` | This site |
