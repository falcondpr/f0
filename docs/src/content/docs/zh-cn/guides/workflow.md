---
title: 任务流程
description: 使用 f0 从工单到 pull request，一步一步来。
sidebar:
  order: 1
---

一个任务的日常流程。前提是已有[项目文件](/zh-cn/reference/configuration/#项目)和 treehouse。

随时确认自己所在的位置：

```bash
pwd                          # 当前目录
git branch --show-current    # 当前分支
f0 ls  # 所有项目的会话和 slot
```

## 1. 开始任务

```bash
cd ~/dev/my-app
f0 new APP-123
```

f0 会运行 `treehouse get --lease --base <base> --branch feature/APP-123`，在新 slot 中打开名为 `my-app/APP-123` 的 tmux 会话，启动编辑器、智能体和每个服务器，并连接到该会话。

分支总是从项目的 `base` 切出。可以用 `--base` 为单个任务覆盖它。

分支类型是项目 `branch_prefixes` 中的第一个，默认是 `feature/`。用 `--type` 为任务选择其他类型：

```bash
f0 new APP-124 --type fix       # fix/APP-124
f0 new APP-125 -t chore         # chore/APP-125
```

会话名仍然是 `my-app/APP-124`。在 `branch_prefixes` 出现之前创建的项目只有 `feature/`，可以用 `f0 edit` 添加其他前缀。

### 多个智能体

一个任务可以运行多个智能体，堆叠在智能体列中。随任务一起启动它们，第一个在最上面：

```bash
f0 new APP-126 --agent claude --agent codex
```

或者在运行中的会话里用 `Ctrl-a A` 添加一个，也可以在任意窗格中执行 [`f0 agent codex`](/zh-cn/reference/cli/#f0-agent)。f0 会为该 slot 记住它们，所以重新打开任务时它们会回来；只有第一个会恢复对话。

它们都在同一个分支上编辑同样的文件。添加智能体可以用来审查第一个写的代码、运行并修复测试，或者在不打断它的情况下回答关于代码的问题。两个并行的修改请开两个任务：每个任务都有自己的 slot 和分支。

## 2. 环境文件

租用 slot 时，treehouse 会把 `.worktreeinclude` 中列出的文件从主检出复制到 slot 中。参见 [Worktree](/zh-cn/guides/worktrees/)。

## 3. 服务器

底部一行已经在运行项目的服务器。如何停止、重启或添加服务器，参见[服务器](/zh-cn/guides/servers/)。

## 4. 审查与验证

用 [diff 视图](/zh-cn/guides/reviewing-changes/)审查智能体的改动，然后在空闲的 shell 中运行检查，例如：

```bash
pnpm nx run-many -t lint test typecheck
```

如果某项检查在基础分支上就已经失败，请在 PR 中注明，而不是绕过它去修复。

## 5. 提交

遵循你团队的约定。一种常见约定：英文、祈使语气、可选的作用域、末尾附上工单号。

```bash
git add -A
git commit -m "[Web] Add login screen (APP-123)"
```

## 6. 创建 pull request

```bash
git push -u origin feature/APP-123
gh pr create --base develop --fill
```

## 7. 完成

在任务会话中按 `Ctrl-a X`（状态栏会提醒你）并选择 **Close and return the slot**，或者在它的任意窗格中输入 `f0 done`。在其他终端中，需要指明任务：

```bash
f0 done APP-123   # 按任务
f0 done my-app/4  # 或者按 `f0 ls` 显示的 slot id
```

会话名（`my-app/APP-123`）或 slot 路径也可以。

f0 会关闭会话并运行 `treehouse return`，它会停止残留的进程，并把 slot 还回池中。treehouse 在丢弃未提交的改动之前会先询问。

如果只想暂停任务，可以在同一个菜单中选择 **Close, keep the slot**，或者运行 `f0 stop`：会话会关闭，slot 仍然归你所有。

之后要回到任务，`f0 open APP-123` 会重新连接；如果会话已关闭，则重建它。重建的会话保留 worktree 及其未提交的改动，智能体会恢复该任务的最后一次对话。
