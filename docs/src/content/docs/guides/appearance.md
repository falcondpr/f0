---
title: Appearance
description: The shared light and dark theme of Ghostty, tmux and Neovim, and how to change it.
sidebar:
  order: 5
---

The bundled configs share one palette, [Catppuccin](https://catppuccin.com): Latte in light mode and Mocha in dark mode. Ghostty, tmux and Neovim follow the macOS appearance, and switch with it while the session is open.

Panels are told apart by background shade, not by lines. The code and the terminals sit on the base color, and the sidebars, the status bars, the pane titles and the floating windows on a slightly darker one. Secondary text is muted, and one accent color (peach) marks the session name, the active pane title and the Neovim mode.

## How each layer follows the system

| Layer | How |
|---|---|
| Ghostty | `theme = light:Catppuccin Latte,dark:Catppuccin Mocha` in `config/ghostty/config` |
| tmux | `config/tmux/appearance.sh` reads the macOS appearance, sets tmux's `theme` (which picks the `dark-theme-*` or `light-theme-*` colors in `config/tmux/tmux.conf`) and paints every pane with the background, text and ANSI colors of the palette. tmux runs it on attach, on focus and every 5 seconds; it needs tmux 3.8 or later |
| Neovim | `config/nvim/lua/f0/appearance.lua` reads the macOS appearance on focus and every 5 seconds and sets `background`; the `catppuccin` colorscheme picks Latte or Mocha from it |

Nothing has to be restarted when the appearance changes: tmux and Neovim switch within a few seconds.

## Other terminals

Since tmux paints the panes itself, the workspace looks the same in any terminal with true color, like WezTerm, iTerm2 or kitty, whatever colors the terminal has. Two things still come from the terminal: the margin around tmux and the cursor color. To match them too, give the terminal the same theme. In WezTerm, for example, in `~/.wezterm.lua`:

```lua
local appearance = wezterm.gui and wezterm.gui.get_appearance() or "Dark"
config.color_scheme = appearance:find("Dark") and "Catppuccin Mocha" or "Catppuccin Latte"
```

## Your own Ghostty config

Ghostty reads more than one config file, and colors set in any of them win over the theme. If the margin or the cursor keep their old colors, remove `background`, `foreground`, `palette` and similar lines from your other Ghostty config files, like `~/Library/Application Support/com.mitchellh.ghostty/config`.

## Changing the theme

Change the layers together, or they stop matching:

- **Ghostty:** another `theme` in `config/ghostty/config`. `ghostty +list-themes` lists the bundled ones.
- **tmux:** the `dark-theme-*` and `light-theme-*` colors in `config/tmux/tmux.conf`, for the bars and the pane titles, and the pane colors in `config/tmux/appearance.sh`. Each color in `tmux.conf` has a role, written next to them: panel shade, selection, muted text and accent.
- **Neovim:** `config/nvim/lua/plugins/colorscheme.lua`, which also holds the overrides that give the panels their shade.

With `install.sh --cli-only`, or `--no-nvim`, `--no-tmux-config` and `--no-ghostty`, your own configs and colors are left alone.
