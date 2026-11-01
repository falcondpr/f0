-- Latte o Mocha según 'background', que nvim toma de la terminal (Ghostty usa el
-- mismo par de temas). Los paneles se distinguen por el tono del fondo, no por
-- líneas: el editor va sobre `base` y lo demás (paneles, floats, separadores) sobre
-- `mantle`. El único color de acento es `peach`.
return {
  "catppuccin/nvim",
  name = "catppuccin",
  lazy = false,
  priority = 1000,
  config = function()
    require("catppuccin").setup({
      flavour = "auto",
      background = { light = "latte", dark = "mocha" },
      float = { solid = true },
      custom_highlights = function(c)
        local muted_title = { fg = c.overlay1, bg = c.mantle }
        return {
          WinSeparator = { fg = c.mantle, bg = c.mantle },
          NeoTreeWinSeparator = { link = "WinSeparator" },
          NeoTreeVertSplit = { link = "WinSeparator" },
          NeoTreeIndentMarker = { fg = c.surface1 },
          NeoTreeExpander = { fg = c.overlay0 },
          FloatTitle = muted_title,
          TelescopeTitle = muted_title,
          TelescopePromptTitle = muted_title,
          TelescopeResultsTitle = muted_title,
          TelescopePreviewTitle = muted_title,
          TelescopeSelection = { fg = c.text, bg = c.surface0 },
          TelescopeSelectionCaret = { fg = c.peach, bg = c.surface0 },
          TelescopePromptPrefix = { fg = c.peach },
          TelescopeMatching = { fg = c.peach },
          F0EmptyKey = { fg = c.overlay2 },
          F0EmptyText = { fg = c.overlay0 },
        }
      end,
    })
    vim.cmd.colorscheme("catppuccin")
  end,
}
