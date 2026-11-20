---
title: 快捷键
description: f0 启动器以及内置 tmux、Ghostty 和 Neovim 配置的快捷键。
sidebar:
  order: 3
---

启动器的按键是 f0 的一部分。其他快捷键来自 `config/tmux`、`config/ghostty` 和 `config/nvim` 中的内置配置。`Space` 是 Neovim 的 leader 键。

## 启动器

用于 f0.app 的主屏幕 [`f0 launch`](/zh-cn/reference/cli/#f0-launch)。

| 按键 | 项目 | 任务 |
|---|---|---|
| 方向键或 `h/j/k/l` | 在卡片之间移动 | 在卡片之间移动 |
| `Enter` | 查看它的任务 | 打开任务并连接到它的会话 |
| `a` | 从你的仓库中添加项目 | |
| `n` | | 开始一个任务，会询问 ticket |
| `c` | | 关闭会话并保留 slot（`f0 stop`） |
| `x` | | 确认后结束任务并归还 slot（`f0 done`） |
| `e` | 编辑项目（`f0 edit`） | 编辑项目（`f0 edit`） |
| `d` | 检查安装（`f0 doctor`） | |
| `t` | 切换主题：自动（跟随 macOS）、浅色或深色。参见[外观](/zh-cn/guides/appearance/#从启动器切换浅色或深色) | |
| `/` | 过滤卡片 | 过滤卡片 |
| `Esc` | 清除过滤 | 清除过滤，然后返回项目列表 |
| `q` | 退出并关闭 f0.app 窗口 | 退出并关闭 f0.app 窗口 |

在 `a` 显示的仓库列表中，`Enter` 添加仓库，`p` 输入一个目录，`Esc` 返回。

## tmux

前缀键是 `Ctrl-a`。

| 按键 | 操作 |
|---|---|
| `Ctrl-h/j/k/l` | 在窗格和 Neovim 分屏之间移动 |
| `Ctrl-a z` | 放大当前窗格（切换） |
| `Ctrl-a s` | 选择会话 |
| `Ctrl-a d` | 分离会话；一切继续运行 |
| `Ctrl-a X` | 关闭此会话的 f0 任务：一个菜单，可以保留 slot 以后再用（`f0 stop`），或在询问后归还它（`f0 done`）。由 f0 自己绑定，参见[按键](/zh-cn/reference/configuration/#按键) |
| `Ctrl-a A` | 在此会话的智能体下方启动另一个智能体：询问命令，直接按 Enter 启动项目的智能体（`f0 agent`）。由 f0 自己绑定 |
| `Ctrl-a \|` / `Ctrl-a -` | 向右 / 向下分割 |
| `Ctrl-a c` | 新建窗口 |
| `Ctrl-a [` | 向上滚动（`v` 选择，`y` 复制到剪贴板） |
| `Ctrl-a Ctrl-l` | 向窗格发送 `Ctrl-l`（清屏） |
| `Ctrl-a Ctrl-k` | 向窗格发送 `Ctrl-k` |
| `Ctrl-a r` | 重新加载 tmux 配置 |

鼠标已启用：点击获得焦点，拖动边框调整大小，滚动查看历史。

`Ctrl-Shift-x` 和 `Ctrl-Enter` 等按键可以到达 Neovim 和智能体：配置为 Ghostty 和 xterm 终端开启了 tmux 的扩展按键（CSI u 格式）。

窗格的 shell 退出时（例如在空提示符下按 `Ctrl-d`），窗格不会关闭：f0 会在同一个窗格中启动一个新的 shell，并在提示符中输入该窗格的命令（`nvim .`、agent、服务器），等待按 Enter。要有意关闭某个窗格，请使用 `Ctrl-a x`。

## Ghostty

这些快捷键在 f0.app 中由它自己的配置提供；在你的 Ghostty 中，则在链接了内置的 `config/ghostty/config` 时生效。

| 按键 | 操作 |
|---|---|
| `Cmd-k` | 清空当前窗格，与 `Ctrl-a Ctrl-l` 相同 |
| `Cmd-d` / `Cmd-Shift-d` | 不做任何事。窗格由 tmux 管理，Ghostty 的分屏会处在 f0 会话之外。仍然可以通过 Ghostty 的菜单分割窗口 |

## Neovim

### 文件与搜索

| 按键 | 操作 |
|---|---|
| `Space ff` | 查找文件 |
| `Space fg` | 在项目中搜索文本 |
| `Space fb` | 已打开的缓冲区 |
| `Space fr` | 最近的文件 |
| `Space e` | 显示或隐藏文件浏览器 |
| `Space gm` | 显示或隐藏已修改文件面板 |
| `Space ui` | 恢复三列布局 |
| `Space un` | 显示或隐藏行号（相对行号），用于 `5j` 这类带计数的移动 |
| `Shift-l` / `Shift-h` | 下一个 / 上一个缓冲区 |
| `Space bd` | 关闭缓冲区；如果有未保存的改动，会询问保存、丢弃或取消 |

### Git

| 按键 | 操作 |
|---|---|
| `]c` / `[c` | 下一处 / 上一处改动 |
| `Space gp` | 预览改动 |
| `Space gr` | 撤销改动 |
| `Space gb` | 查看该行的 blame |
| `Space gs` | 已修改文件（选择器） |
| `Space gc` | 提交记录 |
| `Space gd` | 未提交改动的 diff |
| `Space gD` | 分支与其基础分支的 diff |
| `Space gh` | 文件历史 |
| `Space gq` | 关闭 diff |

### 代码

| 按键 | 操作 |
|---|---|
| `gd` / `gD` | 定义 / 声明 |
| `gi` / `gr` | 实现 / 引用 |
| `K` | 悬停文档 |
| `Space rn` | 重命名 |
| `Space ca` | 代码操作 |
| `[d` / `]d` | 上一个 / 下一个诊断 |
| `Space ld` | 显示诊断 |
| `Space cf` | 格式化 |

## 使用你自己的配置

跨窗格使用 `Ctrl-h/j/k/l` 需要 [vim-tmux-navigator](https://github.com/christoomey/vim-tmux-navigator) 的两部分：tmux 配置中的代码片段，以及 Neovim 中的插件。没有它们时，用你的 tmux 前缀键加方向键在窗格之间移动。只有你的 tmux 配置设置了 `pane-border-status`，窗格标题才会显示。

每当 f0 创建或打开会话时，都会在你自己的前缀键之后绑定 `X` 和 `A`，所以它们适用于任何 tmux 配置，而且无需重新加载。每个 f0 会话的状态栏都会显示它们，比如 `C-b X close task  C-b A new agent`。可以用 [`[keys]`](/zh-cn/reference/configuration/#按键) 换按键或关闭它们。
