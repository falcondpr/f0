---
title: Arquitectura
description: Cómo encajan la CLI de f0, tmux, Neovim y treehouse.
sidebar:
  order: 1
---

f0 es un **PDE (Personal Development Environment)**: configuraciones más una pequeña CLI en Rust que las une. La CLI no guarda estado propio: lee treehouse y tmux, y los maneja.

```
f0 (CLI en Rust)
 ├── treehouse  reservar, listar y devolver slots
 ├── tmux       una sesión por espacio de trabajo, armada panel por panel
 └── git        encontrar el checkout, el repositorio principal y la rama
config/
 ├── nvim       editor, explorador, diffs
 ├── tmux       prefijo, navegación, títulos de paneles
 └── ghostty    fuente y teclado
```

## Qué hace `f0 new`

1. Carga `config.toml` y `projects/*.toml`, y elige el proyecto cuyo `repo` es el checkout principal de la carpeta actual.
2. Ejecuta `treehouse get --lease --lease-holder f0:<sesión> --branch <prefijo><ticket>`, con `--base` e `--include-file` si están configurados. treehouse imprime la ruta del slot.
3. Convierte el proyecto en una lista de paneles y divisiones. Este paso es puro y tiene tests.
4. Crea la sesión de tmux con el tamaño de tu terminal, la divide panel por panel, nombra cada panel y escribe su comando.
5. Guarda la ruta del slot en la sesión, en la opción `@f0_path`. `ls`, `open` y `done` encuentran las sesiones a través de ella.
6. Se conecta, o cambia el cliente si ya estás dentro de tmux.

## Organización del código

| Ruta | Contenido |
|---|---|
| `crates/f0/src/main.rs` | Definición de la línea de comandos |
| `crates/f0/src/commands.rs` | Una función por comando |
| `crates/f0/src/config.rs` | Archivos de configuración, valores por defecto y validación |
| `crates/f0/src/layout.rs` | Descripción pura de la ventana |
| `crates/f0/src/tmux.rs` | Creación y búsqueda de sesiones |
| `crates/f0/src/treehouse.rs` | Llamadas a treehouse |
| `config/` | Configuraciones de Neovim, tmux y Ghostty incluidas |
| `install.sh` | Instalador |
| `docs/` | Este sitio |
