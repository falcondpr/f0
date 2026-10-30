---
title: Worktrees
description: Cómo usa f0 los slots de treehouse y .worktreeinclude.
sidebar:
  order: 2
---

Los agentes de IA funcionan mejor con un checkout por tarea, así dos tareas nunca tocan los mismos archivos ni servidores. f0 delega esto en treehouse, que mantiene un pool de worktrees de git reutilizables llamados **slots**.

## Ciclo de vida de un slot

| Paso | Comando | Qué pasa |
|---|---|---|
| Reservar | `f0 new APP-123` | treehouse entrega un slot libre (o crea uno), hace checkout de una rama nueva desde la base y copia los archivos ignorados |
| Trabajar | `f0 open`, `f0 switch` | El slot sigue reservado mientras trabajas, aunque la sesión esté desconectada |
| Devolver | `f0 done APP-123` o `f0 done my-app/4` | Se detienen los procesos que queden, se limpia el slot y vuelve al pool |

Los slots viven en `~/.treehouse/<repo>-<id>/<slot>/<repo>`. `f0 ls` muestra los slots en uso como `<proyecto>/<slot>`, un id único entre proyectos, con su tarea y su sesión.

## .worktreeinclude

Cada proyecto necesita uno: f0 no empieza una tarea sin él. Un slot nuevo solo tiene los archivos versionados. Los archivos ignorados, como `.env`, se copian desde el checkout principal si coinciden con un patrón de `.worktreeinclude` en la raíz del repositorio, uno por línea:

```
apps/api/.env
apps/web/.env
apps/mobile/.env.development
```

Un archivo vacío es válido y no copia nada. La copia ocurre al reservar el slot. Si cambias un `.env` en el checkout principal, cópialo a mano a los slots en uso o empieza una tarea nueva.

### Sin commitear el archivo

Si el repositorio es compartido y no puedes commitear `.worktreeinclude`, guárdalo en tu carpeta de configuración y apunta el proyecto a él:

```toml
include_file = "~/.config/f0/my-app.worktreeinclude"
```

f0 lo pasa como `treehouse get --include-file`.

## Ejecutar `done` dentro de su propia sesión

Devolver un slot detiene todos sus procesos, incluida la shell que estaría ejecutando `f0 done`. Por eso, dentro de la sesión de la tarea, `f0 done` (o `Ctrl-a X`) pregunta en la barra de estado de tmux y deja el trabajo a un job de tmux en segundo plano, que no pertenece a ningún panel. Ese job no puede responder la pregunta de treehouse sobre cambios sin commitear, así que f0 pregunta antes y te dice cuántos se descartarían.
