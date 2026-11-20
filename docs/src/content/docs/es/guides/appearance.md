---
title: Apariencia
description: El tema claro y oscuro que comparten Ghostty, tmux, Neovim y el agente, y cómo cambiarlo.
sidebar:
  order: 5
---

Los configs incluidos comparten una paleta, en versión clara y oscura: grises neutros sin tinte, un solo color de acento (azul) y unos pocos colores de sintaxis. Ghostty, tmux, Neovim y el agente siguen la apariencia de macOS y cambian con ella sin cerrar la sesión.

## La paleta

Los paneles se distinguen por el tono del fondo, no por líneas. Cada color tiene un rol:

| Rol | Claro | Oscuro | Dónde |
|---|---|---|---|
| Base | `#ffffff` | `#1f1f1f` | El código y las terminales |
| Panel | `#f8f8f8` | `#181818` | Paneles laterales, barras de estado, títulos de los paneles, ventanas flotantes |
| Selección | `#e4e6f1` | `#2a2d3a` | Texto seleccionado, el archivo actual en el explorador, el paréntesis que hace pareja |
| Texto | `#1f1f1f` | `#d4d4d4` | El texto, y el título del panel activo |
| Apagado | `#6e6e6e` | `#8b8b8b` | Texto secundario: comentarios, los demás títulos de paneles, las barras de estado |
| Acento | `#005fb8` | `#4daafc` | El nombre de la sesión, el subrayado del título del panel activo, el modo de Neovim, las coincidencias en los buscadores |

El código usa cinco tonos, para keywords, strings, funciones, números y constantes, y tipos, más los comentarios apagados. Nada va en negrita ni en cursiva. El rojo queda para errores y borrados.

## Menos en pantalla

Nada se muestra dos veces, y lo que era solo decoración no está. Todas las funciones siguen ahí:

- **Números de línea** apagados. `Space un` los muestra, relativos, para movimientos con cuenta como `5j`.
- **Una barra de estado por nivel.** La de Neovim muestra la ruta del archivo y si tiene cambios sin guardar, la cantidad de diagnósticos y `línea:columna`, todo apagado. El modo se ve en la forma del cursor, y en la barra, en el acento, solo fuera del modo normal. La rama y la carpeta están en la barra de tmux.
- **El explorador** es una lista simple: sin íconos de archivo, carpetas con `›` y `⌄`, sin guías de indentación. Las letras de git quedan.
- **Los cambios de git** son una barra fina `▎` en la columna de signos, con el color del tipo de cambio.
- **La línea del cursor** se marca solo en la ventana con foco. Las guías de indentación son un `▏` tenue por nivel.
- **Los títulos de los paneles** van en mayúsculas y apagados; el activo va en el color del texto, subrayado en el acento.
- **La barra de título de Ghostty** queda sobre el color de la ventana.

## Claro u oscuro desde el launcher

Por defecto el espacio de trabajo sigue la apariencia de macOS. En la pantalla de proyectos del launcher, `t` alterna entre **auto** (sigue a macOS), **claro** y **oscuro**; la ayuda de teclas de abajo muestra el actual. La elección se guarda en `~/.config/f0/appearance` y se aplica en todo a la vez:

- el launcher y el margen, la barra de título y el cursor de la ventana de f0.app, al instante;
- tmux, en todas las sesiones, al instante;
- Neovim y Claude Code, en unos segundos, sin reiniciarlos.

Al volver a auto, todo sigue otra vez a macOS. Borrar `~/.config/f0/appearance` hace lo mismo.

## Cómo sigue cada capa al sistema

| Capa | Cómo |
|---|---|
| Ghostty | `theme = light:f0-light,dark:f0-dark` en `config/ghostty/config`. Los temas están en `config/ghostty/themes`, e `install.sh` los linkea en `~/.config/ghostty/themes`. f0.app los lee del repo |
| tmux | `config/tmux/appearance.sh` lee la apariencia de macOS, fija el `theme` de tmux (que elige los colores `dark-theme-*` o `light-theme-*` de `config/tmux/tmux.conf`) y pinta cada panel con el fondo, el texto y los colores ANSI de la paleta. tmux lo corre al conectarse, al recibir el foco y cada 5 segundos; hace falta tmux 3.8 o posterior |
| Neovim | `config/nvim/lua/f0/appearance.lua` lee la apariencia de macOS al recibir el foco y cada 5 segundos y fija `background`; el colorscheme elige la paleta clara u oscura según ese valor |
| Claude Code | f0 lo arranca con `--settings '{"theme":"auto"}'`, así sigue el fondo que tmux pinta en su panel. Solo dentro de f0: tu `theme` de `~/.claude/settings.json` queda como está, y un `--settings` en el comando del agente del proyecto tiene prioridad |

No hay que reiniciar nada cuando cambia la apariencia: tmux, Neovim y Claude Code cambian en unos segundos.

## Otras terminales

Como tmux pinta los paneles por su cuenta, el espacio de trabajo se ve igual en cualquier terminal con true color, como WezTerm, iTerm2 o kitty, tenga los colores que tenga. Dos cosas siguen saliendo de la terminal: el margen alrededor de tmux y el color del cursor. Para que también coincidan, ponle a la terminal los mismos colores. En WezTerm, por ejemplo, en `~/.wezterm.lua`:

```lua
local appearance = wezterm.gui and wezterm.gui.get_appearance() or "Dark"
local dark = appearance:find("Dark")
config.colors = {
  background = dark and "#1f1f1f" or "#ffffff",
  foreground = dark and "#d4d4d4" or "#1f1f1f",
  cursor_bg = dark and "#d4d4d4" or "#1f1f1f",
}
```

## Tu propio config de Ghostty

Ghostty lee más de un archivo de config, y los colores definidos en cualquiera de ellos le ganan al tema. Si el margen o el cursor siguen con sus colores viejos, saca las líneas `background`, `foreground`, `palette` y similares de tus otros archivos de config de Ghostty, como `~/Library/Application Support/com.mitchellh.ghostty/config`. [`f0 doctor`](/es/reference/cli/#f0-doctor) lista esas líneas con su archivo y número de línea, avisa si faltan los temas de f0 (corre `install.sh` de nuevo después de actualizar el repo) y avisa si tmux es anterior a 3.8.

## Cambiar el tema

Cambia las capas juntas, o dejan de coincidir:

- **Ghostty:** `config/ghostty/themes/f0-light` y `f0-dark`, u otro `theme` en `config/ghostty/config`. `ghostty +list-themes` lista los que trae.
- **tmux:** los colores `dark-theme-*` y `light-theme-*` de `config/tmux/tmux.conf`, para las barras y los títulos de los paneles, y los colores de los paneles en `config/tmux/appearance.sh`. Cada color de `tmux.conf` tiene un rol, anotado al lado: tono de panel, selección, texto apagado, texto y acento.
- **Neovim:** `config/nvim/lua/plugins/colorscheme.lua`. Mantiene el plugin `catppuccin` por sus integraciones con el explorador, los buscadores y la vista de diff, y reemplaza sus colores con la paleta de arriba; los roles están anotados al principio del archivo.

Con `install.sh --cli-only`, o `--no-nvim`, `--no-tmux-config` y `--no-ghostty`, tus configs y colores quedan como están.
