---
title: Key bindings
description: tmux and Neovim shortcuts in the bundled configs.
sidebar:
  order: 3
---

These come from the bundled configs in `config/tmux` and `config/nvim`. `Space` is the Neovim leader key.

## tmux

The prefix is `Ctrl-a`.

| Keys | Action |
|---|---|
| `Ctrl-h/j/k/l` | Move between panes and Neovim splits |
| `Ctrl-a z` | Zoom the current pane (toggle) |
| `Ctrl-a s` | Pick a session |
| `Ctrl-a d` | Detach; everything keeps running |
| `Ctrl-a X` | Close the f0 task of this session: a menu to keep its slot for later (`f0 stop`) or return it, after asking (`f0 done`). Bound by f0 itself, see [Keys](/reference/configuration/#keys) |
| `Ctrl-a \|` / `Ctrl-a -` | Split right / below |
| `Ctrl-a c` | New window |
| `Ctrl-a [` | Scroll back (`v` select, `y` copy to the clipboard) |
| `Ctrl-a Ctrl-l` | Send `Ctrl-l` (clear screen) to the pane |
| `Ctrl-a Ctrl-k` | Send `Ctrl-k` to the pane |
| `Ctrl-a r` | Reload the tmux config |

The mouse is enabled: click to focus, drag borders to resize, scroll to see history.

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

f0 binds `X` after your own prefix whenever it creates or opens a session, so it works with any tmux config and needs no reload. The status line of each f0 session shows it, like `C-b X close task`. Pick another key or turn it off with [`[keys] done`](/reference/configuration/#keys).
