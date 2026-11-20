---
title: 配置
description: 全局默认值和项目文件。
sidebar:
  order: 2
---

f0 只读取个人配置，所以共享仓库无需任何改动。你至少需要一个项目；`config.toml` 是可选的。

位置，按顺序取第一个匹配：`$F0_CONFIG_DIR`、`$XDG_CONFIG_HOME/f0`、`~/.config/f0`。

```
~/.config/f0/
├── config.toml          全局默认值
├── ghostty.conf         f0.app 的 Ghostty 配置，由 install.sh 写入
└── projects/
    └── my-app.toml      每个项目一个文件
```

未知的键会被拒绝，这样拼写错误会明确报错。任何修改之后都运行 `f0 doctor`。`f0 init` 会问几个问题并为你写好项目文件，`f0 edit` 以同样的方式修改它。

## 全局默认值

`config.toml` 设置每个工作区使用的内容。项目中省略的部分都会回退到这里。

```toml
[defaults.editor]
cmd = "nvim ."

[defaults.agent]
cmd = "claude"

[defaults.layout]
terminals = 4
terminal_height = 25
agent_width = 35
```

| 键 | 默认值 | 含义 |
|---|---|---|
| `defaults.editor.cmd` | `nvim .` | 编辑器窗格的命令 |
| `defaults.agent.cmd` | `claude` | 智能体窗格的命令 |
| `defaults.layout.terminals` | `4` | 底部一行的窗格数，1 到 6 |
| `defaults.layout.terminal_height` | `25` | 底部一行的高度，占窗口的百分比，10 到 90 |
| `defaults.layout.agent_width` | `35` | 智能体窗格的宽度，占顶部一行的百分比，10 到 90 |

服务器属于项目，这里不允许配置。

## 按键

f0 会在你的 tmux 前缀键之后自己绑定它的 tmux 按键，所以它们适用于任何 tmux 配置，而且无需重新加载。`f0 new` 和 `f0 open` 会绑定它们，`f0 keys` 会为已经在运行的会话绑定。

```toml
[keys]
done = "X"
agent = "A"
```

| 键 | 默认值 | 含义 |
|---|---|---|
| `keys.done` | `"X"` | 在按下它的会话中打开一个关闭任务的菜单：保留 slot（如 `f0 stop`），或归还它（如 `f0 done`）。可以是任何 tmux 按键名（`"Q"`、`"M-x"`）；`false` 表示关闭 |
| `keys.agent` | `"A"` | 在状态栏中询问一个智能体，并在按下它的会话中把它启动在其他智能体下方，如 [`f0 agent`](/zh-cn/reference/cli/#f0-agent)。直接按 Enter 启动项目的智能体。取值与 `keys.done` 相同，且必须是不同的按键 |

每个 f0 会话的状态栏都会显示这些按键，比如 `C-a X close task  C-a A new agent`。f0 从不替换你的 tmux 配置已经绑定到其他功能的按键：`f0 keys` 和 `f0 doctor` 会提示你换一个。

## 启动器

[`f0 launch`](/zh-cn/reference/cli/#f0-launch) 中的 `a` 会在 `~/Desktop`、`~/Documents`、`~/Developer`、`~/code`、`~/dev`、`~/Projects` 和 `~/src` 中查找仓库，不存在的目录会跳过。在 `config.toml` 中添加更多目录，它们也会被搜索。

```toml
[launch]
folders = ["~/work", "/Volumes/Data/repos"]
```

| 键 | 默认值 | 含义 |
|---|---|---|
| `launch.folders` | `[]` | 除默认目录外，额外查找仓库的目录 |

## AI 智能体

智能体窗格运行你给它的任何命令，所以任何能在终端中运行的 AI 编程 CLI 都可以作为智能体：Claude Code（`claude`）、Codex（`codex`）、Gemini CLI（`gemini`）、opencode（`opencode`）等等。窗格以程序命名，比如 `agent: codex`。

| 使用范围 | 设置方式 |
|---|---|
| 所有项目 | 在 `config.toml` 中设置 `[defaults.agent] cmd = "codex"` |
| 单个项目 | 在项目文件中设置 `[agent] cmd = "opencode"`，或在 `f0 init` 和 `f0 edit` 中回答这个问题 |
| 单个任务 | `f0 new APP-123 --agent gemini` |
| 一个任务中的多个智能体 | `f0 new APP-123 --agent claude --agent codex`，或在运行中的会话里执行 [`f0 agent codex`](/zh-cn/reference/cli/#f0-agent) |

可以带参数（`cmd = "codex --full-auto"`）。当智能体的程序没有安装时，`f0 new` 和 `f0 open` 会拒绝启动，`f0 doctor` 会检查每个项目的智能体。

### 恢复对话

当 `f0 open` 重建一个已关闭的会话时（例如重启之后），智能体会接着该任务的最后一次对话。每个任务都有自己的 worktree，所以恢复的总是该任务的对话，而不是其他任务的。Claude Code 通过 `claude --continue` 开箱即用。对于其他智能体，在设置 `cmd` 的地方同时设置用于恢复的命令：

```toml
[agent]
cmd = "codex"
resume = "codex resume --last"
```

一个会话中有多个智能体时，只有第一个会恢复：两个按目录恢复的智能体会接上同一个对话。其他智能体重新开始。

如果恢复命令失败（例如还没有任何对话），智能体会重新开始。`f0 new` 总是开始新的对话。

## 项目

`projects/<name>.toml`。文件名就是项目名称，也是每个会话名的第一部分。f0 只在有项目的仓库中工作。

```toml
repo = "~/dev/my-app"
base = "develop"
branch_prefixes = ["feature/", "fix/", "refactor/", "chore/"]
# include_file = "~/.config/f0/my-app.worktreeinclude"

[[servers]]
name = "api"
cmd = "pnpm nx serve @org/api"

[[servers]]
name = "mobile"
cwd = "apps/mobile"
cmd = "npx expo start -c"

[agent]
cmd = "claude"

[layout]
terminals = 4
```

| 键 | 默认值 | 含义 |
|---|---|---|
| `repo` | 必需 | 主检出。会展开 `~/`。与当前目录匹配，在 slot 内部也可以 |
| `base` | 由 treehouse 推断 | 新任务从哪个分支切出。导出为 `F0_BASE` |
| `branch_prefixes` | `["feature/", "fix/", "refactor/", "chore/"]` | `f0 new` 分支的前缀。第一个是默认值，`f0 new --type fix` 选择 `fix/`。会从会话名中去掉。仍然只有单个 `branch_prefix = "feature/"` 的文件继续可用，只是只有这一个前缀 |
| `include_file` | `<repo>/.worktreeinclude` | 列出要复制到每个 slot 中的被忽略文件的文件。它必须存在；空文件什么也不复制 |
| `servers` | 无 | `name`、`cmd`、可选的 `cwd`（相对于 slot）和 `autostart`（默认 `false`：命令等待 Enter）的列表 |
| `editor`、`agent`、`layout` | 全局默认值 | 与 `config.toml` 中的键相同，只是没有 `defaults.` 前缀 |

服务器的数量必须不超过 `layout.terminals`。

## 会话名

| 场景 | 名称 |
|---|---|
| 任务 | `<project>/<task>`，例如 `my-app/APP-123` |
| HEAD 处于分离状态的 slot | `<project>/slot-<n>` |

`.` 和 `:` 会变成 `_`，因为 tmux 把它们用作分隔符。
