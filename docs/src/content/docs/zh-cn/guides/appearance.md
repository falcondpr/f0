---
title: 外观
description: Ghostty、tmux、Neovim 和智能体共用的浅色与深色主题，以及如何修改它。
sidebar:
  order: 5
---

内置配置共用一套调色板，分为浅色和深色两个版本：不带色调的中性灰、一种强调色（蓝色）和少量语法颜色。Ghostty、tmux、Neovim 和智能体都跟随 macOS 的外观，并在会话打开时随之切换。

## 调色板

面板之间靠背景深浅区分，而不是靠线条。每种颜色都有一个角色：

| 角色 | 浅色 | 深色 | 用在哪里 |
|---|---|---|---|
| 基础 | `#ffffff` | `#1f1f1f` | 代码和终端 |
| 面板 | `#f8f8f8` | `#181818` | 侧边栏、状态栏、窗格标题、浮动窗口 |
| 选区 | `#e4e6f1` | `#2a2d3a` | 选中的文字、文件浏览器中的当前文件、配对的括号 |
| 文字 | `#1f1f1f` | `#d4d4d4` | 文字，以及当前窗格的标题 |
| 淡色 | `#6e6e6e` | `#8b8b8b` | 次要文字：注释、其他窗格标题、状态栏 |
| 强调 | `#005fb8` | `#4daafc` | 会话名、当前窗格标题的下划线、Neovim 模式、选择器中的匹配 |

代码使用五种色调，分别用于关键字、字符串、函数、数字和常量、类型，再加上淡色的注释。没有粗体或斜体。红色只用于错误和删除。

## 屏幕上更少的东西

任何信息都不重复显示，纯装饰的元素都去掉了。所有功能依然可用：

- **行号**默认关闭。`Space un` 会显示相对行号，用于 `5j` 这类带计数的移动。
- **每一层一条状态栏。** Neovim 的状态栏显示文件路径和是否有未保存的修改、诊断数量和 `行:列`，全部为淡色。模式通过光标形状显示；只有在普通模式之外，状态栏才会用强调色显示模式。分支和文件夹在 tmux 状态栏中。
- **文件浏览器**是一个简单的列表：没有文件图标，文件夹用 `›` 和 `⌄`，没有缩进线。git 字母保留。
- **git 修改**在标记列中显示为一条细线 `▎`，颜色表示修改类型。
- **当前行**只在有焦点的窗口中标出。缩进参考线是每级一个浅色的 `▏`。
- **窗格标题**为大写淡色；当前窗格的标题使用文字颜色，并带有强调色下划线。
- **Ghostty 的标题栏**与窗口同色。

## 从启动器切换浅色或深色

默认情况下，工作区跟随 macOS 外观。在启动器的项目界面中，`t` 会在**自动**（跟随 macOS）、**浅色**和**深色**之间切换；底部的按键提示会显示当前模式。选择会保存在 `~/.config/f0/appearance` 中，并同时应用到所有地方：

- 启动器，以及 f0.app 窗口的边距、标题栏和光标，立即生效；
- tmux 的所有会话，立即生效；
- Neovim 和 Claude Code，在几秒内生效，无需重启。

切回自动后，一切重新跟随 macOS。删除 `~/.config/f0/appearance` 效果相同。

## 每一层如何跟随系统

| 层 | 方式 |
|---|---|
| Ghostty | `config/ghostty/config` 中的 `theme = light:f0-light,dark:f0-dark`。主题位于 `config/ghostty/themes`，`install.sh` 会把它们链接到 `~/.config/ghostty/themes`。f0.app 自带一份副本，由 `install.sh` 复制 |
| tmux | `config/tmux/appearance.sh` 读取 macOS 外观，设置 tmux 的 `theme`（它会从 `config/tmux/tmux.conf` 中选择 `dark-theme-*` 或 `light-theme-*` 颜色），并用调色板的背景色、文字色和 ANSI 颜色绘制每个窗格。tmux 会在连接时、获得焦点时以及每 5 秒运行它一次；需要 tmux 3.8 或更高版本 |
| Neovim | `config/nvim/lua/f0/appearance.lua` 在获得焦点时以及每 5 秒读取一次 macOS 外观并设置 `background`；配色方案据此选择浅色或深色调色板 |
| Claude Code | f0 用 `--settings '{"theme":"auto"}'` 启动它，使它跟随 tmux 在其窗格中绘制的背景。仅在 f0 内生效：你在 `~/.claude/settings.json` 中的 `theme` 保持不变，项目智能体命令中自带的 `--settings` 优先 |

外观改变时无需重启任何东西：tmux、Neovim 和 Claude Code 会在几秒内切换。

## 其他终端

由于 tmux 自己绘制窗格，所以在任何支持真彩色的终端中（例如 WezTerm、iTerm2 或 kitty），无论终端本身是什么颜色，工作区看起来都一样。仍有两样东西来自终端：tmux 周围的边距和光标颜色。要让它们也一致，就给终端设置同样的颜色。以 WezTerm 为例，在 `~/.wezterm.lua` 中：

```lua
local appearance = wezterm.gui and wezterm.gui.get_appearance() or "Dark"
local dark = appearance:find("Dark")
config.colors = {
  background = dark and "#1f1f1f" or "#ffffff",
  foreground = dark and "#d4d4d4" or "#1f1f1f",
  cursor_bg = dark and "#d4d4d4" or "#1f1f1f",
}
```

## 你自己的 Ghostty 配置

Ghostty 会读取多个配置文件，其中任何一个设置的颜色都会覆盖主题。如果边距或光标保持旧颜色，就从你的其他 Ghostty 配置文件（例如 `~/Library/Application Support/com.mitchellh.ghostty/config`）中删除 `background`、`foreground`、`palette` 之类的行。[`f0 doctor`](/zh-cn/reference/cli/#f0-doctor) 会列出这些行及其文件和行号，报告缺少的 f0 主题（更新仓库后请重新运行 `install.sh`），并报告低于 3.8 的 tmux 版本。

## 更换主题

各层要一起修改，否则它们就不再匹配：

- **Ghostty：** `config/ghostty/themes/f0-light` 和 `f0-dark`，或在 `config/ghostty/config` 中换一个 `theme`。`ghostty +list-themes` 会列出内置主题。
- **tmux：** `config/tmux/tmux.conf` 中用于状态栏和窗格标题的 `dark-theme-*` 和 `light-theme-*` 颜色，以及 `config/tmux/appearance.sh` 中的窗格颜色。`tmux.conf` 中的每种颜色都有一个角色，写在它们旁边：面板深浅、选区、淡色文字、文字和强调色。
- **Neovim：** `config/nvim/lua/plugins/colorscheme.lua`。它保留 `catppuccin` 插件，以使用它与文件浏览器、选择器和 diff 视图的集成，并用上面的调色板覆盖其颜色；各角色写在文件开头。

使用 `install.sh --cli-only`，或者 `--no-nvim`、`--no-tmux-config` 和 `--no-ghostty` 时，你自己的配置和颜色保持不变。
