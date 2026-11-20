---
title: Worktree
description: f0 如何使用 treehouse slot 和 .worktreeinclude。
sidebar:
  order: 2
---

AI 智能体在每个任务一个检出时效果最好，这样两个任务永远不会碰到相同的文件或服务器。f0 把这件事交给 treehouse，它维护一个可复用的 git worktree 池，其中的每个 worktree 称为 **slot**。

## slot 的生命周期

| 步骤 | 命令 | 发生了什么 |
|---|---|---|
| 租用 | `f0 new APP-123` | treehouse 分配一个空闲 slot（或新建一个），从基础分支检出新分支，并放入被忽略的文件 |
| 工作 | `f0 open`、`f0 switch` | 工作期间 slot 保持租用状态，即使会话已分离 |
| 归还 | `f0 done APP-123` 或 `f0 done my-app/4` | 停止残留的进程，清理 slot 并放回池中 |

slot 位于 `~/.treehouse/<repo>-<id>/<slot>/<repo>`。`f0 ls` 以 `<project>/<slot>` 的形式显示正在使用的 slot，这个 id 在所有项目中唯一，并附带它们的任务和会话。

### 超过 16 个任务

treehouse 每个仓库最多保留 16 个 slot。要同时运行更多任务，在仓库根目录的 `treehouse.toml` 中调高 `max_trees`：

```toml
max_trees = 32
```

如果不想把这个文件提交到仓库，把它加到 `.git/info/exclude`。

## .worktreeinclude

每个项目都需要一个：没有它，f0 不会启动任务。新的 slot 只包含被跟踪的文件。被忽略的文件（例如 `.env`）如果匹配仓库根目录 `.worktreeinclude` 中的模式，就会从主检出中复制过来，每行一个模式：

```
apps/api/.env
apps/web/.env
apps/mobile/.env.development
```

空文件是有效的，什么也不复制。复制发生在租用 slot 时。修改主检出中的 `.env` 之后，需要自己把它复制到正在运行的 slot 中，或者开始一个新任务。

### 不提交该文件

如果仓库是共享的，你无法提交 `.worktreeinclude`，可以把它放在你的配置目录中，并让项目指向它：

```toml
include_file = "~/.config/f0/my-app.worktreeinclude"
```

f0 会把它作为 `treehouse get --include-file` 传入。

## 在自己的会话中运行 `done`

归还 slot 会停止其中的每个进程，包括正在运行 `f0 done` 的 shell。所以在任务自己的会话中，`f0 done`（或 `Ctrl-a X`）会先在 tmux 状态栏中询问，然后把你的终端带出 tmux，在那里完成工作，让你看到输出。那时 treehouse 已经无法询问未提交的改动，所以 f0 会先询问，并告诉你有多少改动会被丢弃。
