---
title: CLI
description: f0 的每个命令和参数。
sidebar:
  order: 1
---

```
f0 <command> [options]
```

运行 `f0 <command> --help` 可以在终端中看到相同的信息。

## f0 launch

显示 f0 的主屏幕：你的项目，然后是你所选项目的任务。[f0.app](/zh-cn/getting-started/installation/#f0app) 在每个窗口中运行它，它也可以在任何其他终端中使用。

```bash
f0 launch
```

项目和任务以卡片形式显示，选中的卡片使用强调色边框。项目卡片显示任务数量；任务卡片显示分支，以及它是运行中（`●`，并显示上次使用时间）还是已停止（`○`：slot 已租用，但没有会话）。所有按键见[快捷键](/zh-cn/reference/keybindings/#启动器)。

- **在任务上按 Enter** 会像 `f0 open` 一样打开它并连接到它的会话。通过 detach（`Ctrl-a d`）、`f0 stop` 或 `f0 done` 离开会话后，会回到该项目的任务列表（已刷新）；按 Esc 返回项目列表。
- **`n`** 询问 ticket，并像 `f0 new` 一样开始任务。
- **`a`**（在项目列表中）在[搜索目录](/zh-cn/reference/configuration/#启动器)中最多向下三层查找尚未成为项目的 git 仓库，跳过 `node_modules`、`target`、隐藏目录和 treehouse slot。在某个仓库上按 Enter 会为它运行 `f0 init` 的问题，`p` 可以输入搜索没有找到的仓库目录。
- **`q`** 退出启动器，这会关闭 f0.app 的窗口。启动器不提供进入独立 shell 的方式。

在 tmux 中运行时，它会拒绝并提示使用 `f0 switch`：在 tmux 窗格中运行启动器会嵌套会话。

## f0 init

为一个仓库创建项目文件，过程中会询问相关信息。

```bash
f0 init [path] [--name <name>] [--template]
```

| 参数 | 含义 |
|---|---|
| `[path]` | 仓库内的某个目录。默认为当前目录 |
| `--name` | 项目名称的默认答案。否则使用仓库目录名 |
| `--template` | 不询问，直接写入带有 `<...>` 占位符的文件 |

它会询问项目名称、基础分支（默认是 `origin/HEAD` 指向的分支，在没有远程仓库的 repo 中则是当前分支）、分支前缀、[AI 智能体](/zh-cn/reference/configuration/#ai-智能体)、env 文件和开发服务器。按 Enter 采用方括号中的值。

```
Project name [react-native-bottom-sheet]: bottom-sheet
Base branch [main]:
Branch prefixes, comma separated, the first is the default [feature/, fix/, refactor/, chore/]:
AI agent command (claude, codex, gemini, opencode...) [claude]:
Other files to copy, comma separated (empty for none):
Add a dev server? [y/N]: y
  Name: example
  Command: bun run example start
  Folder, relative to the repo (empty for the root):
  Start it with every task? [y/N]:
Add another server? [y/N]:
```

当仓库没有 `.worktreeinclude` 时，f0 会列出它找到的被忽略的 `.env*` 文件，把你选中的写入配置目录中的 `<name>.worktreeinclude`，并让 `include_file` 指向它。如果仓库有自己的 `.worktreeinclude`，就继续使用它，f0 会跳过这个问题。当服务器多于终端时，文件中也会调高 `layout.terminals`。

在没有终端的环境中（例如脚本里）或使用 `--template` 时，它会写入占位符版本；只要还剩有占位符，`f0 new`、`f0 open` 和 `f0 doctor` 就会拒绝该项目。`Ctrl-D` 会在写入任何内容之前取消，而且它从不覆盖：如果仓库已经有项目或文件已存在，它会失败。

## f0 edit

逐个问题更新项目文件，并在保存前显示改动。

```bash
f0 edit [project]
```

| 参数 | 含义 |
|---|---|
| `[project]` | 要编辑的项目。默认是你所在仓库的项目 |

它会询问与 `f0 init` 相同的问题，并以当前值作为默认值，所以按 Enter 就会保留。对于每个服务器，你可以选择保留、编辑或删除，之后还可以添加更多。

```
Base branch [develop]:
Branch prefixes, comma separated, the first is the default [feature/, fix/, refactor/, chore/]:
AI agent command (claude, codex, gemini, opencode...) [claude]:
Env files: using the repo's .worktreeinclude
Server api: pnpm nx serve @org/api
  Keep, edit or remove? [K/e/r]: e
  Name [api]:
  Command [pnpm nx serve @org/api]:
  Folder, relative to the repo [.]:
  Start it with every task? [y/N]: y
Server admin: pnpm nx dev @org/admin
  Keep, edit or remove? [K/e/r]: r
Add another server? [y/N]:
```

如果没有任何改动，它会告诉你并保持文件不变。否则它会显示文件的 `git diff`，并在保存前询问；新文件会一步替换旧文件。`editor`、`agent` 和 `layout` 会保留，你自己写的注释则不会。`f0 init --template` 留下的占位符会作为问题让你填写。它需要终端。

## f0 new

租用一个带有新分支的 treehouse slot，并打开它的工作区。

```bash
f0 new <ticket> [--project <name>] [--base <branch>] [--type <type>] [--start <servers>] [--agent <command>]... [--no-attach]
```

| 参数 | 含义 |
|---|---|
| `<ticket>` | 任务 id，追加在项目 `branch_prefixes` 的第一个前缀之后（`APP-123` 变成 `feature/APP-123`） |
| `-p`、`--project` | 项目名称。默认是 `repo` 包含当前目录的那个项目 |
| `--base` | 从哪个分支切出，覆盖项目中的 `base` |
| `-t`、`--type` | 分支类型：项目 `branch_prefixes` 中的一个，带不带 `/` 都可以（`--type fix` 会创建 `fix/APP-123`）。会话名仍然是 `my-app/APP-123` |
| `--start` | 立即启动的服务器，用逗号分隔（`--start api,web`）。其他服务器等待 Enter，除非它们设置了 `autostart = true` |
| `--agent` | 仅用于此任务的 AI 智能体，比如 `codex` 或 `opencode`，覆盖项目中的设置。重复使用可在智能体列中堆叠多个智能体，第一个在最上面：`--agent claude --agent codex` |
| `--no-attach` | 创建会话但不连接，并输出会话名 |

需要 treehouse、项目文件及其 [worktree include 文件](/zh-cn/guides/worktrees/#worktreeinclude)。如果缺少其中任何一个，或者已存在同名会话，会在租用 slot 之前失败。

当分支已经存在时（本地或 `origin` 上），比如一个在合并前就被归还的任务，slot 会检出该分支而不是创建它，`--base` 不起作用。如果另一个检出正在使用该分支，它会失败：如果那是一个 slot，`f0 open` 可以重新打开它。

## f0 open

重新打开一个 treehouse slot 的工作区；如果它的会话正在运行，就连接到该会话。

```bash
f0 open [slot] [--start <servers>] [--agent <command>]... [--no-attach]
```

| 参数 | 含义 |
|---|---|
| `[slot]` | 要打开的 slot：它的 id（`my-app/4`）、任务（`APP-123`）、会话名或路径（其中任意目录都可以），与 `f0 ls` 显示的一致。默认是当前目录 |
| `--start` | 重建会话时要启动的服务器，与 `f0 new` 相同。会话已在运行时不起作用 |
| `--agent` | 重建会话使用的 AI 智能体，与 `f0 new` 相同，替换会话原有的智能体 |
| `--no-attach` | 创建会话但不连接 |

重建的会话会带回关闭时的所有智能体，包括用 [`f0 agent`](#f0-agent) 添加的，其中第一个会恢复它在该任务中的最后一次对话。其他智能体重新开始：按目录恢复（如 `claude --continue`）无法区分它们的对话。参见[恢复对话](/zh-cn/reference/configuration/#恢复对话)。

当任务的会话被关闭后（例如 tmux 或电脑重启后），用它把任务找回来。slot 必须属于一个已配置的项目；其他目录会被拒绝。

已存在的目录总是优先于名称。单独的 slot 编号（`4`）也可以，但它会在多个项目中重复：这时会使用当前目录所在的项目；不在任何项目中时，f0 会列出可选的 id。

## f0 agent

在任务的会话中，在已有智能体的下方启动另一个智能体。

```bash
f0 agent [command]
```

| 参数 | 含义 |
|---|---|
| `[command]` | 智能体及其参数，比如 `codex` 或 `codex --full-auto`。默认是项目的智能体 |

在任务会话的任意窗格中运行它，或者按 `Ctrl-a A`，它会在状态栏中询问命令（直接按 Enter 启动项目的智能体）。新窗格位于最后一个智能体下方，在 slot 的目录中，标题为 `agent: codex`，同一程序的第二个则为 `agent: claude 2`；智能体列保持原宽度，其中的智能体平分高度。在 f0 会话之外运行会失败，不做任何改动。

f0 会记住每个 slot 的智能体，所以 `f0 stop` 和 `f0 open`（或启动器）会把它们带回来。用 `Ctrl-a x` 关闭的智能体窗格不会再回来。用 `f0 done` 归还 slot 时会忘掉它们，该 slot 的下一个任务从项目的智能体开始。

一个会话中的所有智能体都在同一个目录、同一个分支上工作。第二个智能体适合在第一个工作时做审查、测试或回答关于代码的问题；两个并行的修改应该放在两个任务中。

## f0 ls

列出正在使用的 slot，以及它们的任务和会话。

```bash
f0 ls [--all] [--paths]
```

```
ID         TASK        STATUS  SESSION
website/1  (detached)  in-use  website/slot-1
my-app/4   APP-123     leased  my-app/APP-123
Free slots: my-app 2 (`f0 ls --all` lists them)
```

| 参数 | 含义 |
|---|---|
| `--all` | 同时列出空闲的 slot |
| `--paths` | 添加一列，显示每个 slot 的目录 |

treehouse 按仓库为 slot 编号，所以同一个编号会出现在多个项目中。**ID** 列 `<project>/<slot>` 是唯一的，`f0 open` 和 `f0 done` 都接受它。**TASK** 是去掉项目 `branch_prefixes` 中匹配前缀之后的分支名。

## f0 switch

从与 [`f0 launch`](#f0-launch) 相同样式的卡片中选择一个 f0 会话（最近使用的排在前面）并切换过去。在 tmux 中会切换当前客户端；在 tmux 外则连接到该会话。Enter 切换，`/` 过滤，Esc 或 `q` 不切换直接退出。

## f0 stop

关闭 slot 的会话，但保留 slot 及其分支和未提交的改动，以便之后回来。

```bash
f0 stop [slot]
```

接受的 slot 写法与 `f0 done` 相同，默认是当前目录。智能体和服务器会停止；`f0 ls` 仍会列出该 slot，`f0 open APP-123` 会重建它的会话。在要关闭的会话中输入时，会话会立即关闭。

## f0 done

关闭 slot 的会话，并把 slot 归还到池中。

```bash
f0 done [slot] [--force]
```

可以用 `f0 ls` 显示的任意形式指定 slot：

```bash
f0 done my-app/4                             # ID 列：在所有项目中唯一
f0 done APP-123                              # TASK 列
f0 done my-app/APP-123                       # SESSION 列
f0 done ~/.treehouse/my-app-3f9a2c/4/my-app  # 路径，来自 `f0 ls --paths`
```

不指定 `[slot]` 时使用当前目录。先关闭会话，然后 `treehouse return` 清理 slot，在丢弃未提交的改动之前会询问；`--force` 会不经询问直接丢弃。不是 slot 的目录会被拒绝。

在它要关闭的会话中，关闭会话会让 `f0 done` 自己中途停止，所以它会改为在 tmux 状态栏中询问（`Close my-app/APP-123 and return its slot? (y/n)`，如果有会被丢弃的未提交改动，还会显示数量），然后你的终端离开 tmux，在那里关闭会话并归还 slot，让你看到结果。`Ctrl-a X` 在任何窗格中都提供这个操作，旁边还有关闭会话但保留 slot 的选项（`f0 stop`），会话的状态栏会显示这个按键。如果归还 slot 失败，终端会显示原因并等待 Enter，slot 仍会留在 `f0 ls` 中。

## f0 keys

在正在运行的 tmux 服务器上绑定 f0 的按键（关闭任务的按键和启动另一个智能体的按键），并在每个 f0 会话的状态栏中显示它们。

```bash
f0 keys
```

`f0 new` 和 `f0 open` 会为它们的会话执行这一步，`install.sh` 在更新后也会运行它，所以已经打开的会话无需重新加载 tmux 就能获得新的绑定。按键来自 [`[keys]`](/zh-cn/reference/configuration/#按键)；修改某个按键后，运行 `f0 keys` 解除旧按键的绑定。

## f0 doctor

检查工具、全局默认值、项目配置、tmux 按键，以及阻止内置主题生效的因素。缺少必需项时以错误退出。

| 检查项 | 必需 |
|---|---|
| `tmux`、`git`、`treehouse` | 是 |
| 至少一个项目 | 是 |
| 默认值和每个项目中的编辑器与智能体命令 | 是 |
| 每个项目的 `repo` 和 worktree include 文件 | 是 |
| 正在运行的 tmux 服务器绑定了关闭任务和启动另一个智能体的按键（没有服务器，或某个按键设为 `false` 时跳过） | 是 |
| `gh`、`rg`、Ghostty | 否 |
| `/Applications/f0.app`（会区分同名的其他应用）及其配置 `~/.config/f0/ghostty.conf` | 否 |
| 链接了内置 tmux 配置时：tmux 3.8 或更高版本，主题需要它 | 是 |
| 链接了内置 Ghostty 配置时：其他 Ghostty 配置文件中没有 `background`、`foreground`、`palette`、`cursor-*` 或 `selection-*` 行，有的话会列出其文件和行号 | 否 |

## 环境变量

| 变量 | 读取方 | 含义 |
|---|---|---|
| `F0_CONFIG_DIR` | f0 | 配置目录，覆盖默认值 |
| `XDG_CONFIG_HOME` | f0 | 配置目录变为 `$XDG_CONFIG_HOME/f0` |
| `F0_PROJECT` | 窗格 | 在会话的每个窗格中设置：项目名称 |
| `F0_BASE` | 窗格 | 项目设置了 `base` 时设置；Neovim 中的 `Space gD` 会用到 |
