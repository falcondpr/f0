-- Claro u oscuro según 'background', que nvim toma de la terminal (Ghostty usa los
-- mismos valores en config/ghostty/themes). Se sigue usando catppuccin por sus
-- integraciones (neo-tree, Telescope, gitsigns, cmp, diffview), pero con la paleta de
-- f0 encima: grises sin tinte, un solo acento (azul) y cinco tonos de sintaxis.
--
-- Roles de cada nombre de catppuccin:
--   base: el editor. mantle: los paneles (sidebars, barras, floats). crust: más oscuro.
--   surface0: la selección. surface1: guías de indentación y bordes tenues.
--   overlay*: texto apagado (overlay2 es el de los comentarios). text: el texto.
--   blue: el acento, y también las funciones. mauve: keywords. green: strings.
--   peach: números y constantes. yellow: tipos y avisos. red: errores.
-- Los demás tonos de catppuccin se pliegan sobre esos, para no sumar colores.
local function palette(p)
  return {
    rosewater = p.text, flamingo = p.text, lavender = p.text, maroon = p.text, sky = p.text,
    pink = p.mauve, teal = p.green, sapphire = p.blue,
    mauve = p.mauve, green = p.green, blue = p.blue, peach = p.peach, yellow = p.yellow, red = p.red,
    text = p.text, subtext1 = p.subtext1, subtext0 = p.subtext0,
    overlay2 = p.overlay2, overlay1 = p.overlay1, overlay0 = p.overlay0,
    surface2 = p.surface2, surface1 = p.surface1, surface0 = p.surface0,
    base = p.base, mantle = p.mantle, crust = p.crust,
  }
end

local light = palette({
  mauve = "#af00db", green = "#3d7a1f", blue = "#005fb8", peach = "#a35200",
  yellow = "#8a6a00", red = "#c72e2e",
  text = "#1f1f1f", subtext1 = "#333333", subtext0 = "#4d4d4d",
  overlay2 = "#6e6e6e", overlay1 = "#8a8a8a", overlay0 = "#a8a8a8",
  surface2 = "#c4c4c4", surface1 = "#e6e6e6", surface0 = "#e4e6f1",
  base = "#ffffff", mantle = "#f8f8f8", crust = "#eeeeee",
})

local dark = palette({
  mauve = "#c586c0", green = "#8cc265", blue = "#4daafc", peach = "#e5a46e",
  yellow = "#d7ba7d", red = "#f14c4c",
  text = "#d4d4d4", subtext1 = "#bdbdbd", subtext0 = "#a8a8a8",
  overlay2 = "#8b8b8b", overlay1 = "#737373", overlay0 = "#5c5c5c",
  surface2 = "#4a4a4a", surface1 = "#303030", surface0 = "#2a2d3a",
  base = "#1f1f1f", mantle = "#181818", crust = "#141414",
})

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
      no_bold = true,
      no_italic = true,
      color_overrides = { latte = light, mocha = dark },
      custom_highlights = function(c)
        local muted_title = { fg = c.overlay2, bg = c.mantle }
        return {
          WinSeparator = { fg = c.mantle, bg = c.mantle },
          NeoTreeWinSeparator = { link = "WinSeparator" },
          NeoTreeVertSplit = { link = "WinSeparator" },
          NeoTreeExpander = { fg = c.overlay1 },
          NeoTreeDirectoryIcon = { fg = c.overlay1 },
          NeoTreeDirectoryName = { fg = c.text },
          NeoTreeRootName = { fg = c.text },
          NeoTreeCursorLine = { bg = c.surface0 },
          FloatTitle = muted_title,
          TelescopeTitle = muted_title,
          TelescopePromptTitle = muted_title,
          TelescopeResultsTitle = muted_title,
          TelescopePreviewTitle = muted_title,
          TelescopeSelection = { fg = c.text, bg = c.surface0 },
          TelescopeSelectionCaret = { fg = c.blue, bg = c.surface0 },
          TelescopePromptPrefix = { fg = c.blue },
          TelescopeMatching = { fg = c.blue },
          Visual = { bg = c.surface0 },
          VisualNOS = { bg = c.surface0 },
          CursorLine = { bg = c.mantle },
          MatchParen = { bg = c.surface0 },
          Whitespace = { fg = c.surface1 },
          ["@variable.builtin"] = { fg = c.mauve },
          StatusLine = { fg = c.overlay2, bg = c.mantle },
          StatusLineNC = { fg = c.overlay2, bg = c.mantle },
          F0StatusMode = { fg = c.blue, bg = c.mantle },
          F0EmptyKey = { fg = c.subtext0 },
          F0EmptyText = { fg = c.overlay1 },
        }
      end,
    })
    vim.cmd.colorscheme("catppuccin")
  end,
}
