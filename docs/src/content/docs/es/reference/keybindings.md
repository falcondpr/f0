---
title: Atajos
description: Atajos del launcher de f0 y de las configuraciones incluidas de tmux, Ghostty y Neovim.
sidebar:
  order: 3
---

Las teclas del launcher son parte de f0. Las demás vienen de las configuraciones incluidas en `config/tmux`, `config/ghostty` y `config/nvim`. `Space` es la tecla líder de Neovim.

## Launcher

En [`f0 launch`](/es/reference/cli/#f0-launch), la pantalla de inicio de f0.app.

| Teclas | Proyectos | Tareas |
|---|---|---|
| Flechas o `h/j/k/l` | Moverse entre tarjetas | Moverse entre tarjetas |
| `Enter` | Ver sus tareas | Abrir la tarea y conectarse a su sesión |
| `a` | Agregar un proyecto desde tus repositorios | |
| `n` | | Empezar una tarea, pidiendo su ticket |
| `c` | | Cerrar la sesión y conservar el slot (`f0 stop`) |
| `x` | | Terminar la tarea y devolver su slot (`f0 done`), después de preguntar |
| `e` | Editar el proyecto (`f0 edit`) | Editar el proyecto (`f0 edit`) |
| `d` | Revisar la instalación (`f0 doctor`) | |
| `t` | Cambiar el tema: auto (sigue a macOS), claro u oscuro. Ver [Apariencia](/es/guides/appearance/#claro-u-oscuro-desde-el-launcher) | |
| `/` | Filtrar las tarjetas | Filtrar las tarjetas |
| `Esc` | Borrar el filtro | Borrar el filtro, después volver a los proyectos |
| `q` | Salir, cerrando la ventana de f0.app | Salir, cerrando la ventana de f0.app |

En la lista de repositorios que muestra `a`, `Enter` agrega uno, `p` toma una carpeta que escribes y `Esc` vuelve.

## tmux

El prefijo es `Ctrl-a`.

| Teclas | Acción |
|---|---|
| `Ctrl-h/j/k/l` | Moverse entre paneles y ventanas de Neovim |
| `Ctrl-a z` | Ampliar el panel actual (alterna) |
| `Ctrl-a s` | Elegir una sesión |
| `Ctrl-a d` | Desconectarse; todo sigue corriendo |
| `Ctrl-a X` | Cierra la tarea de f0 de esta sesión: un menú para conservar su slot para después (`f0 stop`) o devolverlo, después de preguntar (`f0 done`). La define el propio f0, ver [Teclas](/es/reference/configuration/#teclas) |
| `Ctrl-a \|` / `Ctrl-a -` | Dividir a la derecha / abajo |
| `Ctrl-a c` | Ventana nueva |
| `Ctrl-a [` | Desplazarse hacia atrás (`v` selecciona, `y` copia al portapapeles) |
| `Ctrl-a Ctrl-l` | Envía `Ctrl-l` (limpiar pantalla) al panel |
| `Ctrl-a Ctrl-k` | Envía `Ctrl-k` al panel |
| `Ctrl-a r` | Recarga la configuración de tmux |

El mouse está activado: clic para enfocar, arrastra los bordes para redimensionar, rueda para ver el historial.

Teclas como `Ctrl-Shift-x` y `Ctrl-Enter` llegan a Neovim y al agente: la configuración activa las teclas extendidas de tmux, en el formato CSI u, para las terminales Ghostty y xterm.

Los paneles no se cierran cuando su shell termina, por ejemplo con `Ctrl-d` en un prompt vacío: f0 abre un shell nuevo en el mismo panel, con el comando del panel escrito y esperando Enter (`nvim .`, el agente, el servidor). Para cerrar un panel a propósito, usa `Ctrl-a x`.

## Ghostty

Valen en f0.app, desde su propia configuración, y en tu Ghostty cuando está enlazado el `config/ghostty/config` incluido.

| Teclas | Acción |
|---|---|
| `Cmd-k` | Limpia el panel enfocado, igual que `Ctrl-a Ctrl-l` |
| `Cmd-d` / `Cmd-Shift-d` | No hacen nada. Los paneles son de tmux, y una división de Ghostty quedaría fuera de la sesión de f0. El menú de Ghostty todavía puede dividir la ventana |

## Neovim

### Archivos y búsqueda

| Teclas | Acción |
|---|---|
| `Space ff` | Buscar archivos |
| `Space fg` | Buscar texto en el proyecto |
| `Space fb` | Buffers abiertos |
| `Space fr` | Archivos recientes |
| `Space e` | Muestra u oculta el explorador |
| `Space gm` | Muestra u oculta el panel de cambios |
| `Space ui` | Restaura el layout de tres columnas |
| `Space un` | Muestra u oculta los números de línea (relativos), para movimientos con cuenta como `5j` |
| `Shift-l` / `Shift-h` | Buffer siguiente / anterior |
| `Space bd` | Cierra el buffer; si tiene cambios sin guardar, pregunta si guardar, descartar o cancelar |

### Git

| Teclas | Acción |
|---|---|
| `]c` / `[c` | Cambio siguiente / anterior |
| `Space gp` | Previsualiza el cambio |
| `Space gr` | Deshace el cambio |
| `Space gb` | Blame de la línea |
| `Space gs` | Archivos modificados (buscador) |
| `Space gc` | Commits |
| `Space gd` | Diff de los cambios sin commitear |
| `Space gD` | Diff de la rama contra su base |
| `Space gh` | Historial del archivo |
| `Space gq` | Cierra el diff |

### Código

| Teclas | Acción |
|---|---|
| `gd` / `gD` | Definición / declaración |
| `gi` / `gr` | Implementación / referencias |
| `K` | Documentación al pasar |
| `Space rn` | Renombrar |
| `Space ca` | Acción de código |
| `[d` / `]d` | Diagnóstico anterior / siguiente |
| `Space ld` | Ver diagnóstico |
| `Space cf` | Formatear |

## Con tus propias configuraciones

`Ctrl-h/j/k/l` entre paneles necesita las dos mitades de [vim-tmux-navigator](https://github.com/christoomey/vim-tmux-navigator): su fragmento en tu configuración de tmux y el plugin en Neovim. Sin ellas, muévete entre paneles con tu prefijo de tmux y las flechas. Los títulos de los paneles solo aparecen si tu configuración de tmux define `pane-border-status`.

f0 define `X` después de tu propio prefijo cada vez que crea o abre una sesión, así que funciona con cualquier configuración de tmux y sin recargarla. La barra de estado de cada sesión de f0 la muestra, como `C-b X close task`. Elige otra tecla o desactívala con [`[keys] done`](/es/reference/configuration/#teclas).
