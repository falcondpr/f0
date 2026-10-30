---
title: Capas y decisiones
description: Por qué f0 está dividido en capas y las decisiones detrás.
sidebar:
  order: 2
---

## Capas

| Capa | Necesita | Comandos |
|---|---|---|
| Un worktree por tarea | treehouse, un archivo de proyecto y un `.worktreeinclude` | `f0 new`, `f0 open`, `f0 ls`, `f0 done` |
| Layout de ventanas | tmux | Lo arman `f0 new` y `f0 open` para cada slot |
| Configuraciones de editor, tmux y terminal | nada, se instalan si las pides | opciones de `install.sh` |

Las dos primeras van siempre juntas. Las configuraciones son opcionales: f0 arma el mismo layout con tu propio editor y tu tmux.

## Decisiones

**Un comando por tarea, un worktree por tarea.** Cada espacio de trabajo es un slot de treehouse. Una tarea nunca comparte archivos, ramas ni servidores con otra, y el agente puede trabajar en segundo plano sin tocar tu checkout principal.

**Los archivos env son parte de la tarea.** Un slot sin sus archivos `.env` no puede levantar sus servidores, así que cada proyecto debe listarlos en `.worktreeinclude`, aunque la lista esté vacía.

**Reutilizar treehouse en lugar de manejar worktrees.** treehouse ya resuelve el pool de slots, las reservas, la limpieza de procesos y `.worktreeinclude`. Rehacerlo sobre `git worktree` duplicaría lógica y errores.

**tmux maneja los procesos, el editor maneja los archivos.** Los servidores y el agente viven en paneles de tmux, no en terminales del editor. Sobreviven a cerrar el editor o la terminal, y cada tarea se puede desconectar y retomar.

**Sin estado propio.** f0 lee `treehouse status --json` y las sesiones de tmux cada vez, así nada se desincroniza.

**Reservas, no subshells.** `treehouse get --lease` no es interactivo y mantiene el slot reservado hasta devolverlo, aunque su sesión esté desconectada.

**Los comandos se escriben en shells.** Detener un servidor con `Ctrl-C` deja una shell para reiniciarlo.

**Los servidores te esperan.** Un proyecto lista lo que una tarea puede levantar; tú arrancas lo que necesita con Enter, `--start` o `autostart`. Levantar todo un monorepo en cada tarea gasta memoria y hace pelear a las tareas por los puertos.

**Solo configuración personal.** Los archivos de proyecto viven en tu carpeta de configuración, así los repositorios compartidos no necesitan cambios.

**Las sesiones nacen con su tamaño final.** Los tamaños de los paneles son porcentajes de la ventana con la que se crea la sesión, así que f0 la crea con el tamaño de tu terminal en lugar de redimensionarla después.

**Nunca forzar una devolución.** treehouse pregunta antes de descartar trabajo sin commitear, y f0 mantiene esa pregunta.
