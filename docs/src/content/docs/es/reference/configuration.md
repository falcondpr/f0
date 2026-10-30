---
title: Configuración
description: Valores globales y archivos de proyecto.
sidebar:
  order: 2
---

f0 solo lee configuración personal, así que los repositorios compartidos no necesitan cambios. Necesitas al menos un proyecto; `config.toml` es opcional.

Ubicación, gana la primera que exista: `$F0_CONFIG_DIR`, `$XDG_CONFIG_HOME/f0`, `~/.config/f0`.

```
~/.config/f0/
├── config.toml          valores globales
└── projects/
    └── my-app.toml      un archivo por proyecto
```

Las claves desconocidas se rechazan, así un error de tipeo falla en el momento. Ejecuta `f0 doctor` después de cada cambio. `f0 init` hace unas preguntas y escribe un archivo de proyecto por ti, y `f0 edit` lo cambia de la misma forma.

## Valores globales

`config.toml` define lo que usa cada espacio de trabajo. Los proyectos lo toman para cualquier sección que no definan.

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

| Clave | Por defecto | Significado |
|---|---|---|
| `defaults.editor.cmd` | `nvim .` | Comando del panel de editor |
| `defaults.agent.cmd` | `claude` | Comando del panel del agente |
| `defaults.layout.terminals` | `4` | Paneles de la fila de abajo, de 1 a 6 |
| `defaults.layout.terminal_height` | `25` | Alto de la fila de abajo, en porcentaje de la ventana, de 10 a 90 |
| `defaults.layout.agent_width` | `35` | Ancho del panel del agente, en porcentaje de la fila de arriba, de 10 a 90 |

Los servidores pertenecen a los proyectos y no se permiten aquí.

## Agente de IA

El panel del agente ejecuta el comando que le des, así que cualquier CLI de IA para programar que funcione en una terminal puede ser el agente: Claude Code (`claude`), Codex (`codex`), Gemini CLI (`gemini`), opencode (`opencode`), etc. El panel lleva el nombre del programa, como `agent: codex`.

| Para usarlo | Configura |
|---|---|
| En todos los proyectos | `[defaults.agent] cmd = "codex"` en `config.toml` |
| En un proyecto | `[agent] cmd = "opencode"` en su archivo, o responde la pregunta de `f0 init` y `f0 edit` |
| En una tarea | `f0 new APP-123 --agent gemini` |

Se pueden pasar argumentos (`cmd = "codex --full-auto"`). `f0 new` y `f0 open` no arrancan si el programa del agente no está instalado, y `f0 doctor` revisa el agente de cada proyecto.

### Retomar la conversación

Cuando `f0 open` rearma una sesión cerrada, por ejemplo después de reiniciar, el agente retoma la última conversación de la tarea. Cada tarea tiene su propio worktree, así que siempre retoma la conversación de esa tarea y no otra. Claude Code lo hace sin configurar nada, con `claude --continue`. Para otros agentes, configura el comando que retoma, en el mismo lugar que `cmd`:

```toml
[agent]
cmd = "codex"
resume = "codex resume --last"
```

Si el comando para retomar falla, como cuando todavía no hay conversación, el agente arranca de cero. `f0 new` siempre arranca una conversación nueva.

## Proyectos

`projects/<nombre>.toml`. El nombre del archivo es el nombre del proyecto y la primera parte de cada nombre de sesión. f0 solo funciona en repositorios que tienen un proyecto.

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

| Clave | Por defecto | Significado |
|---|---|---|
| `repo` | obligatoria | Checkout principal. Se expande `~/`. Se compara con la carpeta actual, también desde dentro de un slot |
| `base` | la que infiere treehouse | Rama desde la que se cortan las tareas nuevas. Se exporta como `F0_BASE` |
| `branch_prefixes` | `["feature/", "fix/", "refactor/", "chore/"]` | Prefijos de las ramas de `f0 new`. El primero es el de por defecto, `f0 new --type fix` usa `fix/`. Se quitan del nombre de sesión. Los archivos que todavía tienen un solo `branch_prefix = "feature/"` siguen funcionando, solo con ese prefijo |
| `include_file` | `<repo>/.worktreeinclude` | Archivo con los archivos ignorados que se copian a cada slot. Tiene que existir; si está vacío no copia nada |
| `servers` | ninguno | Lista de `name`, `cmd`, `cwd` opcional (relativo al slot) y `autostart` (por defecto `false`: el comando espera tu Enter) |
| `editor`, `agent`, `layout` | valores globales | Las mismas claves que en `config.toml`, sin el prefijo `defaults.` |

La cantidad de servidores tiene que entrar en `layout.terminals`.

## Nombres de sesión

| Dónde | Nombre |
|---|---|
| Tarea | `<proyecto>/<tarea>`, por ejemplo `my-app/APP-123` |
| Slot sin rama | `<proyecto>/slot-<n>` |

`.` y `:` se convierten en `_` porque tmux los usa como separadores.
