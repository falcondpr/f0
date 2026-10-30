---
title: Servidores
description: Ejecuta, reinicia y organiza servidores de desarrollo en la fila de terminales.
sidebar:
  order: 3
---

Cada entrada `[[servers]]` de un proyecto tiene su propio panel en la fila de abajo, en orden. Los paneles restantes hasta `layout.terminals` son shells libres.

```toml
[[servers]]
name = "api"
cmd = "pnpm nx serve @org/api"

[[servers]]
name = "mobile"
cwd = "apps/mobile"
cmd = "npx expo start -c"
```

- `name` se muestra en el borde del panel.
- `cwd` es relativo al slot.
- El comando se escribe en una shell, no se ejecuta como proceso del panel. `Ctrl-C` detiene el servidor y deja la shell abierta, así que `Arriba` y `Enter` lo reinician.

## Elegir qué arranca

La lista es lo que una tarea *puede* levantar, no lo que levanta. Por defecto no arranca ningún servidor: cada panel abre con su comando escrito en la línea, esperando tu Enter. La mayoría de las tareas toca una o dos apps de un monorepo, así que levantar todo gastaría memoria y haría pelear a los puertos.

| Para | Haz |
|---|---|
| Arrancar un servidor ahora | Aprieta Enter en su panel |
| Arrancar algunos con la tarea | `f0 new APP-123 --start api,web` |
| Arrancar uno en todas las tareas | `autostart = true` en su bloque `[[servers]]` |
| Arrancar servidores al rearmar una sesión | `f0 open APP-123 --start api` |

```toml
[[servers]]
name = "api"
cmd = "pnpm nx serve @org/api"
autostart = true
```

`--start` revisa los nombres antes de reservar el slot, así un error de tipeo falla enseguida y muestra los servidores del proyecto.

## A tener en cuenta

- Un servidor ejecuta el código de la carpeta donde arrancó. Los servidores de otra tarea siguen sirviendo el código de esa tarea.
- Dos tareas con el mismo servidor van a pelear por el mismo puerto. Detén uno o configura puertos distintos por tarea.
- Amplía un panel con `Ctrl-a z` para leer logs largos y desplázate hacia atrás con `Ctrl-a [`.
- `layout.terminals` tiene que ser al menos la cantidad de servidores (máximo 6).
