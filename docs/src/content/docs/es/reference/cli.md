---
title: CLI
description: Todos los comandos y opciones de f0.
sidebar:
  order: 1
---

```
f0 <comando> [opciones]
```

Ejecuta `f0 <comando> --help` para ver la misma información en la terminal (en inglés).

## f0 launch

Muestra la pantalla de inicio de f0: tus proyectos, y después las tareas del que elijas. [f0.app](/es/getting-started/installation/#f0app) lo abre en cada ventana, y también funciona en cualquier otra terminal.

```bash
f0 launch
```

Los proyectos y las tareas se muestran como tarjetas, y la seleccionada lleva el borde del color de acento. La tarjeta de un proyecto cuenta sus tareas; la de una tarea muestra su rama y si está corriendo (`●`, con cuándo la usaste por última vez) o parada (`○`: el slot está alquilado, sin sesión). Mira [Atajos](/es/reference/keybindings/#launcher) para todas las teclas.

- **Enter sobre una tarea** la abre como `f0 open` y se conecta a su sesión. Al salir de la sesión, desconectándote (`Ctrl-a d`), con `f0 stop` o con `f0 done`, vuelve la lista de tareas de ese proyecto, actualizada; Esc sube a los proyectos.
- **`n`** pide un ticket y empieza la tarea como `f0 new`.
- **`a`**, en los proyectos, busca repositorios de git que todavía no son proyectos, hasta tres carpetas de profundidad en las [carpetas de búsqueda](/es/reference/configuration/#launcher), salteando `node_modules`, `target`, las carpetas ocultas y los slots de treehouse. Enter sobre uno le hace las preguntas de `f0 init`, y `p` toma la carpeta de uno que la búsqueda no encontró.
- **`q`** sale del launcher, y eso cierra la ventana de f0.app. El launcher no tiene forma de salir a una shell suelta.

Si se ejecuta dentro de tmux, se niega y sugiere `f0 switch`: un launcher en un panel de tmux anidaría sesiones.

## f0 init

Crea el archivo de proyecto de un repositorio, haciéndote preguntas.

```bash
f0 init [ruta] [--name <nombre>] [--template]
```

| Argumento | Significado |
|---|---|
| `[ruta]` | Una carpeta dentro del repositorio. Por defecto, el directorio actual |
| `--name` | Respuesta por defecto para el nombre del proyecto. Si no, el nombre de la carpeta del repositorio |
| `--template` | Escribe un archivo con marcadores `<...>` en lugar de preguntar |

Pregunta el nombre del proyecto, la rama base (por defecto, a la que apunta `origin/HEAD`, o la rama actual en un repo sin remoto), los prefijos de rama, el [agente de IA](/es/reference/configuration/#agente-de-ia), los archivos env y los servidores de desarrollo. Con Enter aceptas el valor entre corchetes.

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

Si el repositorio no tiene `.worktreeinclude`, f0 lista los archivos `.env*` ignorados que encuentra y escribe los que elijas en `<nombre>.worktreeinclude`, en la carpeta de config, y apunta `include_file` a él. Si el repositorio tiene su propio `.worktreeinclude`, lo sigue usando y f0 no pregunta. Con más servidores que terminales, el archivo también sube `layout.terminals`.

Sin una terminal, como en un script, o con `--template`, escribe la versión con marcadores; `f0 new`, `f0 open` y `f0 doctor` rechazan un proyecto mientras le quede un marcador. `Ctrl-D` cancela antes de escribir nada, y nunca sobrescribe: falla si el repositorio ya tiene un proyecto o si el archivo existe.

## f0 edit

Actualiza un archivo de proyecto pregunta por pregunta y muestra los cambios antes de guardar.

```bash
f0 edit [proyecto]
```

| Argumento | Significado |
|---|---|
| `[proyecto]` | Proyecto a editar. Por defecto, el del repositorio en el que estás |

Hace las mismas preguntas que `f0 init`, con los valores actuales por defecto, así Enter los conserva. Para cada servidor eliges conservarlo, editarlo o quitarlo, y después puedes agregar más.

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

Si nada cambió lo dice y no toca el archivo. Si no, muestra un `git diff` del archivo y pregunta antes de guardar; el archivo nuevo reemplaza al viejo en un solo paso. Conserva `editor`, `agent` y `layout`, pero no los comentarios que hayas escrito a mano. Los marcadores de `f0 init --template` aparecen como preguntas para completar. Necesita una terminal.

## f0 new

Reserva un slot de treehouse con una rama nueva y abre su espacio de trabajo.

```bash
f0 new <ticket> [--project <nombre>] [--base <rama>] [--type <tipo>] [--start <servidores>] [--agent <comando>]... [--no-attach]
```

| Argumento | Significado |
|---|---|
| `<ticket>` | Id de la tarea, se agrega al primero de los `branch_prefixes` del proyecto (`APP-123` pasa a ser `feature/APP-123`) |
| `-p`, `--project` | Nombre del proyecto. Por defecto, el proyecto cuyo `repo` contiene la carpeta actual |
| `--base` | Rama de la que se corta, en lugar de la `base` del proyecto |
| `-t`, `--type` | Tipo de rama: uno de los `branch_prefixes` del proyecto, con o sin la `/` (`--type fix` crea `fix/APP-123`). La sesión sigue siendo `my-app/APP-123` |
| `--start` | Servidores que arrancan enseguida, separados por coma (`--start api,web`). Los demás esperan tu Enter, salvo que tengan `autostart = true` |
| `--agent` | Agente de IA solo para esta tarea, como `codex` u `opencode`, en lugar del del proyecto. Repítelo para apilar varios agentes en la columna del agente, el primero arriba: `--agent claude --agent codex` |
| `--no-attach` | Crea la sesión sin conectarse e imprime su nombre |

Necesita treehouse, un archivo de proyecto y su [archivo de includes](/es/guides/worktrees/#worktreeinclude). Si falta alguno falla antes de reservar el slot, y también si ya existe una sesión con el mismo nombre.

Si la rama ya existe, en local o en `origin`, como con una tarea devuelta antes de mergearla, el slot la usa en vez de crearla, y `--base` no hace nada. Falla si otra copia de trabajo tiene esa rama: `f0 open` la vuelve a abrir si es un slot.

## f0 open

Vuelve a abrir el espacio de trabajo de un slot de treehouse, o se conecta a su sesión si sigue abierta.

```bash
f0 open [slot] [--start <servidores>] [--agent <comando>]... [--no-attach]
```

| Argumento | Significado |
|---|---|
| `[slot]` | Slot a abrir: su id (`my-app/4`), su tarea (`APP-123`), el nombre de su sesión o su ruta (sirve cualquier carpeta dentro), como los muestra `f0 ls`. Por defecto, la carpeta actual |
| `--start` | Servidores que arrancan al rearmar la sesión, como en `f0 new`. No hace nada si la sesión ya está abierta |
| `--agent` | Agentes de IA para la sesión rearmada, como en `f0 new`, en lugar de los que tenía |
| `--no-attach` | Crea la sesión sin conectarse |

Una sesión rearmada vuelve con los agentes que tenía al cerrarse, incluidos los que se agregaron con [`f0 agent`](#f0-agent), y el primero retoma su última conversación en la tarea. Los demás arrancan de cero: retomar por carpeta, como `claude --continue`, no distingue sus conversaciones. Mira [Retomar la conversación](/es/reference/configuration/#retomar-la-conversación).

Sirve para recuperar una tarea cuando su sesión se cerró, por ejemplo si se reinició tmux o la computadora. El slot tiene que pertenecer a un proyecto configurado; cualquier otra carpeta se rechaza.

Una carpeta que existe siempre gana sobre un nombre. El número de slot solo (`4`) también sirve, pero se repite entre proyectos: en ese caso se usa el proyecto de la carpeta actual, y fuera de cualquier proyecto f0 lista los ids para elegir.

## f0 agent

Arranca otro agente en la sesión de una tarea, debajo de los que ya tiene.

```bash
f0 agent [comando]
```

| Argumento | Significado |
|---|---|
| `[comando]` | El agente y sus argumentos, como `codex` o `codex --full-auto`. Por defecto, el agente del proyecto |

Ejecútalo en cualquier panel de la sesión de una tarea, o aprieta `Ctrl-a A`, que pide el comando en la barra de estado (Enter solo arranca el agente del proyecto). El panel nuevo va debajo del último agente, en la carpeta del slot, con el título `agent: codex`, o `agent: claude 2` para un segundo del mismo programa; la columna del agente mantiene su ancho y sus agentes se reparten la altura en partes iguales. Fuera de una sesión de f0 falla sin cambiar nada.

f0 recuerda los agentes de cada slot, así que `f0 stop` y `f0 open`, o el launcher, los traen de vuelta. Un panel de agente cerrado con `Ctrl-a x` queda cerrado. Devolver el slot con `f0 done` los olvida, y la próxima tarea en ese slot arranca con el agente del proyecto.

Todos los agentes de una sesión trabajan en la misma carpeta y en la misma rama. Un segundo agente sirve para revisar, probar o responder preguntas sobre el código mientras el primero trabaja; dos cambios en paralelo van en dos tareas.

## f0 ls

Lista los slots en uso, con su tarea y su sesión.

```bash
f0 ls [--all] [--paths]
```

```
ID         TASK        STATUS  SESSION
website/1  (detached)  in-use  website/slot-1
my-app/4   APP-123     leased  my-app/APP-123
Free slots: my-app 2 (`f0 ls --all` lists them)
```

| Argumento | Significado |
|---|---|
| `--all` | También lista los slots libres |
| `--paths` | Agrega una columna con la carpeta de cada slot |

treehouse numera los slots por repositorio, así que el mismo número aparece en varios proyectos. La columna **ID**, `<proyecto>/<slot>`, es única, y `f0 open` y `f0 done` la aceptan. **TASK** es la rama sin el prefijo de `branch_prefixes` con el que empiece.

## f0 switch

Elige una sesión de f0 entre tarjetas como las de [`f0 launch`](#f0-launch), las usadas más recientemente primero, y cambia a ella. Dentro de tmux cambia el cliente actual; fuera, se conecta. Enter cambia, `/` filtra, y Esc o `q` salen sin cambiar.

## f0 stop

Cierra la sesión de un slot y conserva el slot, con su rama y sus cambios sin commitear, para volver más tarde.

```bash
f0 stop [slot]
```

Recibe el slot como `f0 done`, y por defecto usa la carpeta actual. El agente y los servidores se detienen; `f0 ls` sigue listando el slot, y `f0 open APP-123` rearma su sesión. Escrito dentro de la sesión que cierra, la sesión se cierra en el momento.

## f0 done

Cierra la sesión de un slot y devuelve el slot al pool.

```bash
f0 done [slot] [--force]
```

Nombra el slot de cualquiera de las formas que muestra `f0 ls`:

```bash
f0 done my-app/4                             # la columna ID: única entre proyectos
f0 done APP-123                              # la columna TASK
f0 done my-app/APP-123                       # la columna SESSION
f0 done ~/.treehouse/my-app-3f9a2c/4/my-app  # la ruta, de `f0 ls --paths`
```

Sin `[slot]` usa la carpeta actual. Primero cierra la sesión y después `treehouse return` limpia el slot, y pregunta antes de descartar cambios sin commitear; con `--force` los descarta sin preguntar. Rechaza carpetas que no son un slot.

Dentro de la sesión que cerraría, cerrar la sesión detendría al propio `f0 done` a mitad de camino, así que pregunta en la barra de estado de tmux (`Close my-app/APP-123 and return its slot? (y/n)`, con la cantidad de cambios sin commitear que descartaría, si hay) y después tu terminal sale de tmux y ahí cierra la sesión y devuelve el slot, así ves el resultado. `Ctrl-a X` lo ofrece desde cualquier panel, junto a cerrar la sesión conservando el slot (`f0 stop`), y la barra de estado de la sesión muestra la tecla. Si la devolución falla, la terminal muestra por qué y espera Enter, y el slot sigue en `f0 ls`.

## f0 keys

Define las teclas de f0, la que cierra una tarea y la que arranca otro agente, en el servidor de tmux en ejecución y las muestra en la barra de estado de cada sesión de f0.

```bash
f0 keys
```

`f0 new` y `f0 open` lo hacen para sus sesiones, e `install.sh` lo ejecuta después de actualizar, así que las sesiones que ya estaban abiertas reciben las teclas nuevas sin recargar tmux. Las teclas salen de [`[keys]`](/es/reference/configuration/#teclas); después de cambiar una, ejecuta `f0 keys` para quitar la anterior.

## f0 doctor

Revisa herramientas, valores globales, proyectos, las teclas de tmux y lo que impide que se aplique el tema incluido. Termina con error si falta algo obligatorio.

| Chequeo | Obligatorio |
|---|---|
| `tmux`, `git`, `treehouse` | Sí |
| Al menos un proyecto | Sí |
| Comandos de editor y agente de los valores globales y de cada proyecto | Sí |
| `repo` y archivo de includes de cada proyecto | Sí |
| El servidor de tmux en ejecución define las teclas que cierran una tarea y arrancan otro agente (se omite sin servidor, o para una tecla en `false`) | Sí |
| `gh`, `rg`, Ghostty, JetBrainsMono Nerd Font | No |
| `/Applications/f0.app`, distinguiéndola de otra app con ese nombre, y su configuración `~/.config/f0/ghostty.conf` | No |
| Con la config de tmux incluida enlazada: tmux 3.8 o posterior, que el tema necesita | Sí |
| Con la config de Ghostty incluida enlazada: ninguna línea `background`, `foreground`, `palette`, `cursor-*` o `selection-*` en otros archivos de config de Ghostty, que se listan con su archivo y línea | No |

## Variables de entorno

| Variable | La lee | Significado |
|---|---|---|
| `F0_CONFIG_DIR` | f0 | Carpeta de configuración, reemplaza la de por defecto |
| `XDG_CONFIG_HOME` | f0 | La carpeta de configuración pasa a ser `$XDG_CONFIG_HOME/f0` |
| `F0_PROJECT` | paneles | Presente en cada panel de una sesión: el nombre del proyecto |
| `F0_BASE` | paneles | Presente si el proyecto tiene `base`; la usa `Space gD` en Neovim |
