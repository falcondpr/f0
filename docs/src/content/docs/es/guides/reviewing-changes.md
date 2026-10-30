---
title: Revisar cambios
description: Inspecciona lo que cambió el agente con el panel de cambios y los diffs lado a lado.
sidebar:
  order: 4
---

La configuración de Neovim incluida está pensada para revisar los cambios que hace un agente en el panel de al lado.

## La vista por defecto

`nvim .` abre tres columnas: **archivos modificados** a la izquierda, el archivo en el centro y el **explorador** a la derecha. Los archivos que edita el agente se recargan solos cuando vuelves al editor.

| Teclas | Acción |
|---|---|
| `Space gm` | Muestra u oculta el panel de cambios |
| `Space e` | Muestra u oculta el explorador |
| `Space ui` | Restaura las tres columnas |

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
