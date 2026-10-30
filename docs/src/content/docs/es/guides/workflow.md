---
title: Flujo de una tarea
description: Del ticket al pull request con f0, paso a paso.
sidebar:
  order: 1
---

El flujo diario de una tarea. Supone un [archivo de proyecto](/es/reference/configuration/#proyectos) y treehouse.

Para ubicarte en cualquier momento:

```bash
pwd                          # carpeta actual
git branch --show-current    # rama actual
f0 ls  # sesiones y slots de cada proyecto
```

## 1. Empezar la tarea

```bash
cd ~/dev/my-app
f0 new APP-123
```

f0 ejecuta `treehouse get --lease --base <base> --branch feature/APP-123`, abre una sesión de tmux llamada `my-app/APP-123` dentro del slot nuevo, arranca el editor, el agente y cada servidor, y se conecta a ella.

La rama siempre sale de la `base` del proyecto. Usa `--base` para cambiarla en una tarea puntual.

## 2. Archivos de entorno

treehouse copia los archivos listados en `.worktreeinclude` desde el checkout principal al slot cuando lo reserva. Mira [Worktrees](/es/guides/worktrees/).

## 3. Servidores

La fila de abajo ya corre los servidores del proyecto. Mira [Servidores](/es/guides/servers/) para detener, reiniciar o agregar uno.

## 4. Revisar y verificar

Revisa lo que cambió el agente con la [vista de diffs](/es/guides/reviewing-changes/) y después ejecuta tus chequeos en una shell libre, por ejemplo:

```bash
pnpm nx run-many -t lint test typecheck
```

Si un chequeo ya falla en la rama base, anótalo en el PR en lugar de esquivarlo.

## 5. Commit

Sigue la convención de tu equipo. Una habitual: en inglés, en imperativo, con scope opcional y el ticket al final.

```bash
git add -A
git commit -m "[Web] Add login screen (APP-123)"
```

## 6. Abrir el pull request

```bash
git push -u origin feature/APP-123
gh pr create --base develop --fill
```

## 7. Terminar

Desde una terminal que no esté dentro de la sesión de la tarea:

```bash
f0 done APP-123   # por tarea
f0 done my-app/4  # o por el id del slot que muestra `f0 ls`
```

También sirven el nombre de la sesión (`my-app/APP-123`) o la ruta del slot.

f0 cierra la sesión y ejecuta `treehouse return`, que detiene los procesos que queden y devuelve el slot al pool. treehouse pregunta antes de descartar cambios sin commitear.

Para retomar una tarea más tarde, `f0 open APP-123` se reconecta, o rearma la sesión si estaba cerrada. La sesión rearmada conserva el worktree con sus cambios sin commitear, y el agente retoma la última conversación de la tarea.
