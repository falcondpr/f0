---
title: Key bindings
description: Shortcuts of the f0 launcher and of the bundled tmux, Ghostty and Neovim configs.
sidebar:
  order: 3
---

The launcher keys are part of f0. The others come from the bundled configs in `config/tmux`, `config/ghostty` and `config/nvim`. `Space` is the Neovim leader key.

## Launcher

In [`f0 launch`](/reference/cli/#f0-launch), the home screen of f0.app.

| Keys | Projects | Tasks |
|---|---|---|
| Arrows or `h/j/k/l` | Move between cards | Move between cards |
| `Enter` | Show its tasks | Open the task and attach to its session |
| `a` | Add a project from your repositories | |
| `n` | | Start a task, asking for its ticket |
| `c` | | Close the session and keep the slot (`f0 stop`) |
| `x` | | Finish the task and return its slot (`f0 done`), after asking |
| `e` | Edit the project (`f0 edit`) | Edit the project (`f0 edit`) |
| `d` | Check the setup (`f0 doctor`) | |
| `t` | Switch the theme: auto (follows macOS), light or dark. See [Appearance](/guides/appearance/#light-or-dark-from-the-launcher) | |
| `/` | Filter the cards | Filter the cards |
| `Esc` | Clear the filter | Clear the filter, then back to the projects |
| `q` | Quit, closing the f0.app window | Quit, closing the f0.app window |

In the list of repositories that `a` shows, `Enter` adds one, `p` takes a folder you type, and `Esc` goes back.

## tmux

The prefix is `Ctrl-a`.

| Keys | Action |
|---|---|
| `Ctrl-h/j/k/l` | Move between panes and Neovim splits |
| `Ctrl-a z` | Zoom the current pane (toggle) |
| `Ctrl-a s` | Pick a session |
| `Ctrl-a d` | Detach; everything keeps running |
| `Ctrl-a X` | Close the f0 task of this session: a menu to keep its slot for later (`f0 stop`) or return it, after asking (`f0 done`). Bound by f0 itself, see [Keys](/reference/configuration/#keys) |
| `Ctrl-a A` | Start another agent under the ones of this session: asks for the command, Enter alone starts the project's (`f0 agent`). Bound by f0 itself |
| `Ctrl-a \|` / `Ctrl-a -` | Split right / below |
| `Ctrl-a c` | New window |
| `Ctrl-a [` | Scroll back (`v` select, `y` copy to the clipboard) |
| `Ctrl-a Ctrl-l` | Send `Ctrl-l` (clear screen) to the pane |
| `Ctrl-a Ctrl-k` | Send `Ctrl-k` to the pane |
| `Ctrl-a r` | Reload the tmux config |

The mouse is enabled: click to focus, drag borders to resize, scroll to see history.

Keys like `Ctrl-Shift-x` and `Ctrl-Enter` reach Neovim and the agent: the config turns on tmux's extended keys, in the CSI u format, for Ghostty and xterm terminals.

Panes never close when their shell exits, for example with `Ctrl-d` at an empty prompt: f0 starts a new shell in the same pane, with the pane's command typed and waiting for Enter (`nvim .`, the agent, the server). To close a pane on purpose, use `Ctrl-a x`.

## Ghostty

These apply in f0.app, from its own config, and in your Ghostty when the bundled `config/ghostty/config` is linked.

| Keys | Action |
|---|---|
| `Cmd-k` | Clear the focused pane, same as `Ctrl-a Ctrl-l` |
| `Cmd-d` / `Cmd-Shift-d` | Do nothing. The panes belong to tmux, and a Ghostty split would sit outside the f0 session. Ghostty's menu can still split the window |

## Neovim

### Files and search

| Keys | Action |
|---|---|
| `Space ff` | Find files |
| `Space fg` | Search text in the project |
| `Space fb` | Open buffers |
| `Space fr` | Recent files |
| `Space e` | Toggle the explorer |
| `Space gm` | Toggle the changed files panel |
| `Space ui` | Restore the three-column layout |
| `Space un` | Toggle line numbers (relative), for counted motions like `5j` |
| `Shift-l` / `Shift-h` | Next / previous buffer |
| `Space bd` | Close the buffer, asking to save, discard or cancel if it has unsaved changes |

### Git

| Keys | Action |
|---|---|
| `]c` / `[c` | Next / previous change |
| `Space gp` | Preview the change |
| `Space gr` | Undo the change |
| `Space gb` | Blame the line |
| `Space gs` | Changed files (picker) |
| `Space gc` | Commits |
| `Space gd` | Diff of uncommitted changes |
| `Space gD` | Diff of the branch against its base |
| `Space gh` | File history |
| `Space gq` | Close the diff |

### Code

| Keys | Action |
|---|---|
| `gd` / `gD` | Definition / declaration |
| `gi` / `gr` | Implementation / references |
| `K` | Hover documentation |
| `Space rn` | Rename |
| `Space ca` | Code action |
| `[d` / `]d` | Previous / next diagnostic |
| `Space ld` | Show diagnostic |
| `Space cf` | Format |

## With your own configs

`Ctrl-h/j/k/l` across panes needs both halves of [vim-tmux-navigator](https://github.com/christoomey/vim-tmux-navigator): its snippet in your tmux config and the plugin in Neovim. Without them, move between panes with your tmux prefix and the arrow keys. Pane titles appear only if your tmux config sets `pane-border-status`.

f0 binds `X` and `A` after your own prefix whenever it creates or opens a session, so they work with any tmux config and need no reload. The status line of each f0 session shows them, like `C-b X close task  C-b A new agent`. Pick other keys or turn them off with [`[keys]`](/reference/configuration/#keys).
