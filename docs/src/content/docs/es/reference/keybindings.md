---
title: Atajos
description: Atajos de tmux y Neovim en las configuraciones incluidas.
sidebar:
  order: 3
---

Vienen de las configuraciones incluidas en `config/tmux` y `config/nvim`. `Space` es la tecla líder de Neovim.

## tmux

El prefijo es `Ctrl-a`.

| Teclas | Acción |
|---|---|
| `Ctrl-h/j/k/l` | Moverse entre paneles y ventanas de Neovim |
| `Ctrl-a z` | Ampliar el panel actual (alterna) |
| `Ctrl-a s` | Elegir una sesión |
| `Ctrl-a d` | Desconectarse; todo sigue corriendo |
| `Ctrl-a \|` / `Ctrl-a -` | Dividir a la derecha / abajo |
| `Ctrl-a c` | Ventana nueva |
| `Ctrl-a [` | Desplazarse hacia atrás (`v` selecciona, `y` copia al portapapeles) |
| `Ctrl-a Ctrl-l` | Envía `Ctrl-l` (limpiar pantalla) al panel |
| `Ctrl-a Ctrl-k` | Envía `Ctrl-k` al panel |
| `Ctrl-a r` | Recarga la configuración de tmux |

El mouse está activado: clic para enfocar, arrastra los bordes para redimensionar, rueda para ver el historial.

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
