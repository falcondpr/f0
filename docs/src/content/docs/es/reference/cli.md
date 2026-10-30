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

## f0 init

Crea el archivo de proyecto del repositorio en el que estás, haciéndote preguntas.

```bash
f0 init [--name <nombre>] [--template]
```

| Argumento | Significado |
|---|---|
| `--name` | Respuesta por defecto para el nombre del proyecto. Si no, el nombre de la carpeta del repositorio |
| `--template` | Escribe un archivo con marcadores `<...>` en lugar de preguntar |

Pregunta el nombre del proyecto, la rama base (por defecto, a la que apunta `origin/HEAD`, o la rama actual en un repo sin remoto), el prefijo de rama, el [agente de IA](/es/reference/configuration/#agente-de-ia), los archivos env y los servidores de desarrollo. Con Enter aceptas el valor entre corchetes.

```
Project name [react-native-bottom-sheet]: bottom-sheet
Base branch [main]:
Branch prefix [feature/]:
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
Branch prefix [feature/]:
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
f0 new <ticket> [--project <nombre>] [--base <rama>] [--start <servidores>] [--agent <comando>] [--no-attach]
```

| Argumento | Significado |
|---|---|
| `<ticket>` | Id de la tarea, se agrega al `branch_prefix` del proyecto (`APP-123` pasa a ser `feature/APP-123`) |
| `-p`, `--project` | Nombre del proyecto. Por defecto, el proyecto cuyo `repo` contiene la carpeta actual |
| `--base` | Rama de la que se corta, en lugar de la `base` del proyecto |
| `--start` | Servidores que arrancan enseguida, separados por coma (`--start api,web`). Los demás esperan tu Enter, salvo que tengan `autostart = true` |
| `--agent` | Agente de IA solo para esta tarea, como `codex` u `opencode`, en lugar del del proyecto |
| `--no-attach` | Crea la sesión sin conectarse e imprime su nombre |

Necesita treehouse, un archivo de proyecto y su [archivo de includes](/es/guides/worktrees/#worktreeinclude). Si falta alguno falla antes de reservar el slot, y también si ya existe una sesión con el mismo nombre.

## f0 open

Vuelve a abrir el espacio de trabajo de un slot de treehouse, o se conecta a su sesión si sigue abierta.

```bash
f0 open [slot] [--start <servidores>] [--agent <comando>] [--no-attach]
```

| Argumento | Significado |
|---|---|
| `[slot]` | Slot a abrir: su id (`my-app/4`), su tarea (`APP-123`), el nombre de su sesión o su ruta (sirve cualquier carpeta dentro), como los muestra `f0 ls`. Por defecto, la carpeta actual |
| `--start` | Servidores que arrancan al rearmar la sesión, como en `f0 new`. No hace nada si la sesión ya está abierta |
| `--agent` | Agente de IA para la sesión rearmada, como en `f0 new` |
| `--no-attach` | Crea la sesión sin conectarse |

Sirve para recuperar una tarea cuando su sesión se cerró, por ejemplo si se reinició tmux o la computadora. El slot tiene que pertenecer a un proyecto configurado; cualquier otra carpeta se rechaza.

Una carpeta que existe siempre gana sobre un nombre. El número de slot solo (`4`) también sirve, pero se repite entre proyectos: en ese caso se usa el proyecto de la carpeta actual, y fuera de cualquier proyecto f0 lista los ids para elegir.

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

treehouse numera los slots por repositorio, así que el mismo número aparece en varios proyectos. La columna **ID**, `<proyecto>/<slot>`, es única, y `f0 open` y `f0 done` la aceptan. **TASK** es la rama sin el `branch_prefix` del proyecto.

## f0 switch

Muestra una lista numerada de sesiones de f0 y cambia a la que elijas. Dentro de tmux cambia el cliente actual; fuera, se conecta.

## f0 done

Cierra la sesión de un slot y devuelve el slot al pool.

```bash
f0 done [slot]
```

Nombra el slot de cualquiera de las formas que muestra `f0 ls`:

```bash
f0 done my-app/4                             # la columna ID: única entre proyectos
f0 done APP-123                              # la columna TASK
f0 done my-app/APP-123                       # la columna SESSION
f0 done ~/.treehouse/my-app-3f9a2c/4/my-app  # la ruta, de `f0 ls --paths`
```

Sin `[slot]` usa la carpeta actual. Primero cierra la sesión y después `treehouse return` limpia el slot, y pregunta antes de descartar cambios sin commitear. Rechaza carpetas que no son un slot y no se ejecuta desde dentro de la sesión que cerraría.

## f0 doctor

Revisa herramientas, valores globales y proyectos. Termina con error si falta algo obligatorio.

| Chequeo | Obligatorio |
|---|---|
| `tmux`, `git`, `treehouse` | Sí |
| Al menos un proyecto | Sí |
| Comandos de editor y agente de los valores globales y de cada proyecto | Sí |
| `repo` y archivo de includes de cada proyecto | Sí |
| `gh`, `rg`, Ghostty | No |

## Variables de entorno

| Variable | La lee | Significado |
|---|---|---|
| `F0_CONFIG_DIR` | f0 | Carpeta de configuración, reemplaza la de por defecto |
| `XDG_CONFIG_HOME` | f0 | La carpeta de configuración pasa a ser `$XDG_CONFIG_HOME/f0` |
| `F0_PROJECT` | paneles | Presente en cada panel de una sesión: el nombre del proyecto |
| `F0_BASE` | paneles | Presente si el proyecto tiene `base`; la usa `Space gD` en Neovim |
