---
title: Apariencia
description: El tema claro y oscuro que comparten Ghostty, tmux y Neovim, y cómo cambiarlo.
sidebar:
  order: 5
---

Los configs incluidos comparten una paleta, [Catppuccin](https://catppuccin.com): Latte en modo claro y Mocha en modo oscuro. Ghostty, tmux y Neovim siguen la apariencia de macOS y cambian con ella sin cerrar la sesión.

Los paneles se distinguen por el tono del fondo, no por líneas. El código y las terminales van sobre el color base, y los paneles laterales, las barras de estado, los títulos de los paneles y las ventanas flotantes sobre uno apenas más oscuro. El texto secundario va apagado, y un solo color de acento (peach) marca el nombre de la sesión, el título del panel activo y el modo de Neovim.

## Cómo sigue cada capa al sistema

| Capa | Cómo |
|---|---|
| Ghostty | `theme = light:Catppuccin Latte,dark:Catppuccin Mocha` en `config/ghostty/config` |
| tmux | `config/tmux/appearance.sh` lee la apariencia de macOS, fija el `theme` de tmux (que elige los colores `dark-theme-*` o `light-theme-*` de `config/tmux/tmux.conf`) y pinta cada panel con el fondo, el texto y los colores ANSI de la paleta. tmux lo corre al conectarse, al recibir el foco y cada 5 segundos; hace falta tmux 3.8 o posterior |
| Neovim | `config/nvim/lua/f0/appearance.lua` lee la apariencia de macOS al recibir el foco y cada 5 segundos y fija `background`; el colorscheme `catppuccin` elige Latte o Mocha según ese valor |

No hay que reiniciar nada cuando cambia la apariencia: tmux y Neovim cambian en unos segundos.

## Otras terminales

Como tmux pinta los paneles por su cuenta, el espacio de trabajo se ve igual en cualquier terminal con true color, como WezTerm, iTerm2 o kitty, tenga los colores que tenga. Dos cosas siguen saliendo de la terminal: el margen alrededor de tmux y el color del cursor. Para que también coincidan, ponle a la terminal el mismo tema. En WezTerm, por ejemplo, en `~/.wezterm.lua`:

```lua
local appearance = wezterm.gui and wezterm.gui.get_appearance() or "Dark"
config.color_scheme = appearance:find("Dark") and "Catppuccin Mocha" or "Catppuccin Latte"
```

## Tu propio config de Ghostty

Ghostty lee más de un archivo de config, y los colores definidos en cualquiera de ellos le ganan al tema. Si el margen o el cursor siguen con sus colores viejos, saca las líneas `background`, `foreground`, `palette` y similares de tus otros archivos de config de Ghostty, como `~/Library/Application Support/com.mitchellh.ghostty/config`.

## Cambiar el tema

Cambia las capas juntas, o dejan de coincidir:

- **Ghostty:** otro `theme` en `config/ghostty/config`. `ghostty +list-themes` lista los que trae.
- **tmux:** los colores `dark-theme-*` y `light-theme-*` de `config/tmux/tmux.conf`, para las barras y los títulos de los paneles, y los colores de los paneles en `config/tmux/appearance.sh`. Cada color de `tmux.conf` tiene un rol, anotado al lado: tono de panel, selección, texto apagado y acento.
- **Neovim:** `config/nvim/lua/plugins/colorscheme.lua`, donde también están los ajustes que les dan su tono a los paneles.

Con `install.sh --cli-only`, o `--no-nvim`, `--no-tmux-config` y `--no-ghostty`, tus configs y colores quedan como están.
