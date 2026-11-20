---
title: Reviewing changes
description: Inspect what the agent changed with the changed files panel and side-by-side diffs.
sidebar:
  order: 4
---

The bundled Neovim config is built to review changes made by an agent in the next pane.

## The default view

`nvim .` opens the **changed files** on the left and the file next to them. The **explorer** opens in the same place, one panel at a time, so the code and the agent keep their width. The changed files panel updates as the agent writes, and files edited by the agent reload automatically when you focus the editor.

| Keys | Action |
|---|---|
| `Space gm` | Toggle the changed files panel |
| `Space e` | Toggle the explorer, in place of the changed files |
| `Space ui` | Back to the changed files panel |

## Side-by-side diffs

| Keys | Action |
|---|---|
| `Space gd` | Uncommitted changes |
| `Space gD` | The whole branch against `origin/$F0_BASE` (the project's `base`) |
| `Space gh` | History of the current file |
| `Space gq` | Close the diff |

Inside the diff, `Tab` and `Shift-Tab` move between files, `]c` and `[c` jump between changes, and `Enter` on the file list opens that file.

## Long lines

Lines never wrap, and both sides of a diff scroll together horizontally as well as vertically:

| Keys | Action |
|---|---|
| `zl` / `zh` | One column right / left |
| `zL` / `zH` | Half a screen right / left |
| `$` / `0` | End / start of the line |

Trackpad and mouse horizontal scrolling work too.

## Zoom

The diff gets narrow next to the agent. Press `Ctrl-a z` to zoom the editor pane to the full window, and again to restore it.
