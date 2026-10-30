---
title: Task workflow
description: From ticket to pull request with f0, step by step.
sidebar:
  order: 1
---

The everyday flow for one task. It assumes a [project file](/reference/configuration/#projects) and treehouse.

To check where you are at any time:

```bash
pwd                          # current folder
git branch --show-current    # current branch
f0 ls  # sessions and slots of every project
```

## 1. Start the task

```bash
cd ~/dev/my-app
f0 new APP-123
```

f0 runs `treehouse get --lease --base <base> --branch feature/APP-123`, opens a tmux session named `my-app/APP-123` inside the new slot, starts the editor, the agent and every server, and attaches to it.

The branch is always cut from the project's `base`. Use `--base` to override it for one task.

## 2. Environment files

treehouse copies the files listed in `.worktreeinclude` from the main checkout into the slot when it is leased. See [Worktrees](/guides/worktrees/).

## 3. Servers

The bottom row already runs the project's servers. See [Servers](/guides/servers/) to stop, restart or add one.

## 4. Review and verify

Review what the agent changed with the [diff view](/guides/reviewing-changes/), then run your checks in a free shell, for example:

```bash
pnpm nx run-many -t lint test typecheck
```

If a check already fails on the base branch, note it in the PR instead of fixing around it.

## 5. Commit

Use your team's convention. A common one: English, imperative mood, optional scope, ticket at the end.

```bash
git add -A
git commit -m "[Web] Add login screen (APP-123)"
```

## 6. Open the pull request

```bash
git push -u origin feature/APP-123
gh pr create --base develop --fill
```

## 7. Finish

From a terminal that is not inside the task session:

```bash
f0 done APP-123   # by task
f0 done my-app/4  # or by the slot id that `f0 ls` shows
```

The session name (`my-app/APP-123`) or the slot path work too.

f0 closes the session and runs `treehouse return`, which stops leftover processes and gives the slot back to the pool. treehouse asks before discarding uncommitted changes.

To come back to a task later, `f0 open APP-123` reattaches, or rebuilds the session if it was closed.
