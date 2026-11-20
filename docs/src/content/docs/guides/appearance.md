---
title: Appearance
description: The shared light and dark theme of Ghostty, tmux, Neovim and the agent, and how to change it.
sidebar:
  order: 5
---

The bundled configs share one palette, in a light and a dark version: neutral greys with no tint, one accent color (blue) and a handful of syntax colors. Ghostty, tmux, Neovim and the agent follow the macOS appearance, and switch with it while the session is open.

## The palette

Panels are told apart by background shade, not by lines. Each color has one role:

| Role | Light | Dark | Where |
|---|---|---|---|
| Base | `#ffffff` | `#1f1f1f` | The code and the terminals |
| Panel | `#f8f8f8` | `#181818` | Sidebars, status bars, pane titles, floating windows |
| Selection | `#e4e6f1` | `#2a2d3a` | Selected text, the current file in the explorer, the matching bracket |
| Text | `#1f1f1f` | `#d4d4d4` | Text, and the title of the active pane |
| Muted | `#6e6e6e` | `#8b8b8b` | Secondary text: comments, the other pane titles, the status bars |
| Accent | `#005fb8` | `#4daafc` | The session name, the underline of the active pane title, the Neovim mode, the matches in the pickers |

Code uses five hues, for keywords, strings, functions, numbers and constants, and types, plus muted comments. Nothing is bold or italic. Red is kept for errors and deletions.

## Less on screen

Nothing is shown twice, and what is only decoration is gone. Every feature is still there:

- **Line numbers** are off. `Space un` shows them, relative, for counted motions like `5j`.
- **One status line per level.** Neovim's shows the file path and whether it has unsaved changes, the diagnostics count and `line:col`, all muted. The mode is shown by the cursor shape, and in the status line, in the accent, only outside normal mode. The branch and the folder are in the tmux bar.
- **The explorer** is a plain list: no file icons, folders with `›` and `⌄`, no indent guides. The git letters stay.
- **Git changes** are a thin `▎` in the sign column, colored by the kind of change.
- **The current line** is marked only in the focused window. Indent guides are a faint `▏` per level.
- **Pane titles** are uppercase and muted; the active one is in the text color, underlined in the accent.
- **Ghostty's titlebar** sits on the window color.

## Light or dark from the launcher

By default the workspace follows the macOS appearance. On the projects screen of the launcher, `t` switches between **auto** (follow macOS), **light** and **dark**; the key hint at the bottom shows the current one. The choice is saved in `~/.config/f0/appearance` and applies everywhere at once:

- the launcher and the margin, titlebar and cursor of the f0.app window, right away;
- tmux, in every session, right away;
- Neovim and Claude Code, within a few seconds, without restarting them.

Back on auto, everything follows macOS again. Deleting `~/.config/f0/appearance` does the same.

## How each layer follows the system

| Layer | How |
|---|---|
| Ghostty | `theme = light:f0-light,dark:f0-dark` in `config/ghostty/config`. The themes are in `config/ghostty/themes`, and `install.sh` links them into `~/.config/ghostty/themes`. f0.app carries its own copy, made by `install.sh` |
| tmux | `config/tmux/appearance.sh` reads the macOS appearance, sets tmux's `theme` (which picks the `dark-theme-*` or `light-theme-*` colors in `config/tmux/tmux.conf`) and paints every pane with the background, text and ANSI colors of the palette. tmux runs it on attach, on focus and every 5 seconds; it needs tmux 3.8 or later |
| Neovim | `config/nvim/lua/f0/appearance.lua` reads the macOS appearance on focus and every 5 seconds and sets `background`; the colorscheme picks the light or dark palette from it |
| Claude Code | f0 starts it with `--settings '{"theme":"auto"}'`, so it follows the background tmux paints in its pane. Only inside f0: your `theme` in `~/.claude/settings.json` is left as it is, and a `--settings` in the project's agent command wins |

Nothing has to be restarted when the appearance changes: tmux, Neovim and Claude Code switch within a few seconds.

## Other terminals

Since tmux paints the panes itself, the workspace looks the same in any terminal with true color, like WezTerm, iTerm2 or kitty, whatever colors the terminal has. Two things still come from the terminal: the margin around tmux and the cursor color. To match them too, give the terminal the same colors. In WezTerm, for example, in `~/.wezterm.lua`:

```lua
local appearance = wezterm.gui and wezterm.gui.get_appearance() or "Dark"
local dark = appearance:find("Dark")
config.colors = {
  background = dark and "#1f1f1f" or "#ffffff",
  foreground = dark and "#d4d4d4" or "#1f1f1f",
  cursor_bg = dark and "#d4d4d4" or "#1f1f1f",
}
```

## Your own Ghostty config

Ghostty reads more than one config file, and colors set in any of them win over the theme. If the margin or the cursor keep their old colors, remove `background`, `foreground`, `palette` and similar lines from your other Ghostty config files, like `~/Library/Application Support/com.mitchellh.ghostty/config`. [`f0 doctor`](/reference/cli/#f0-doctor) lists those lines with their file and line number, reports missing f0 themes (run `install.sh` again after updating the repo), and reports a tmux older than 3.8.

## Changing the theme

Change the layers together, or they stop matching:

- **Ghostty:** `config/ghostty/themes/f0-light` and `f0-dark`, or another `theme` in `config/ghostty/config`. `ghostty +list-themes` lists the bundled ones.
- **tmux:** the `dark-theme-*` and `light-theme-*` colors in `config/tmux/tmux.conf`, for the bars and the pane titles, and the pane colors in `config/tmux/appearance.sh`. Each color in `tmux.conf` has a role, written next to them: panel shade, selection, muted text, text and accent.
- **Neovim:** `config/nvim/lua/plugins/colorscheme.lua`. It keeps the `catppuccin` plugin for its integrations with the explorer, the pickers and the diff view, and overrides its colors with the palette above; the roles are written at the top of the file.

With `install.sh --cli-only`, or `--no-nvim`, `--no-tmux-config` and `--no-ghostty`, your own configs and colors are left alone.
