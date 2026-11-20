---
title: Revisar cambios
description: Inspecciona lo que cambió el agente con el panel de cambios y los diffs lado a lado.
sidebar:
  order: 4
---

La configuración de Neovim incluida está pensada para revisar los cambios que hace un agente en el panel de al lado.

## La vista por defecto

`nvim .` abre los **archivos modificados** a la izquierda y el archivo al lado. El **explorador** se abre en el mismo lugar, un panel a la vez, así el código y el agente conservan su ancho. El panel de archivos modificados se actualiza mientras el agente escribe, y los archivos que edita se recargan solos cuando vuelves al editor.

| Teclas | Acción |
|---|---|
| `Space gm` | Muestra u oculta el panel de cambios |
| `Space e` | Muestra u oculta el explorador, en lugar del panel de cambios |
| `Space ui` | Vuelve al panel de cambios |

## Diffs lado a lado

| Teclas | Acción |
|---|---|
| `Space gd` | Cambios sin commitear |
| `Space gD` | Toda la rama contra `origin/$F0_BASE` (la `base` del proyecto) |
| `Space gh` | Historial del archivo actual |
| `Space gq` | Cierra el diff |

Dentro del diff, `Tab` y `Shift-Tab` pasan de un archivo a otro, `]c` y `[c` saltan entre cambios, y `Enter` en la lista abre ese archivo.

## Líneas largas

Las líneas nunca se parten, y los dos lados del diff se desplazan juntos tanto en vertical como en horizontal:

| Teclas | Acción |
|---|---|
| `zl` / `zh` | Una columna a la derecha / izquierda |
| `zL` / `zH` | Media pantalla a la derecha / izquierda |
| `$` / `0` | Final / inicio de la línea |

También funciona el desplazamiento horizontal del trackpad y del mouse.

## Ampliar

El diff queda angosto al lado del agente. Presiona `Ctrl-a z` para ampliar el panel del editor a toda la ventana, y otra vez para restaurarlo.
