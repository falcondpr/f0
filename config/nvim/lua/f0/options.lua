local opt = vim.opt

opt.path:append("**")
opt.wildmenu = true
opt.wildignore:append({
  "*/node_modules/*",
  "*/ios/Pods/*",
  "*/.git/*",
  "*/android/build/*",
})

opt.number = true
opt.relativenumber = true
opt.scrolloff = 8

-- Las lineas largas no se parten: siguen de largo y la ventana scrollea en horizontal
opt.wrap = false
opt.sidescroll = 1     -- scroll horizontal de a 1 columna, no de a media pantalla
opt.sidescrolloff = 8  -- columnas de contexto al llegar al borde
opt.ignorecase = true
opt.smartcase = true
opt.termguicolors = true

opt.signcolumn = "yes"
opt.expandtab = true
opt.shiftwidth = 2
opt.tabstop = 2
opt.smartindent = true
opt.undofile = true
opt.updatetime = 250

-- El agente edita archivos desde otro panel: recargarlos en vez de quedar con buffers viejos
opt.autoread = true

-- En un diff, los dos lados scrollean juntos tambien en horizontal (lineas largas)
opt.scrollopt = { "ver", "hor", "jump" }
opt.diffopt:append({ "algorithm:histogram", "indent-heuristic" })
