-- Una sola barra sobre el tono de los paneles, sin separadores. Solo el modo usa el
-- color de acento; el resto va apagado.
local function theme()
  local c = require("catppuccin.palettes").get_palette()
  local muted = { fg = c.overlay1, bg = c.mantle }
  local section = {
    a = { fg = c.peach, bg = c.mantle, gui = "bold" },
    b = muted, c = muted, x = muted, y = muted, z = muted,
  }
  return {
    normal = section,
    insert = section,
    visual = section,
    replace = section,
    command = section,
    terminal = section,
    inactive = { a = muted, b = muted, c = muted },
  }
end

local function setup()
  require("lualine").setup({
    options = {
      theme = theme(),
      globalstatus = true,
      section_separators = "",
      component_separators = "",
    },
    sections = {
      lualine_a = { "mode" },
      lualine_b = { "branch" },
      lualine_c = { { "filename", path = 1, symbols = { unnamed = "" } } },
      lualine_x = { "diagnostics" },
      lualine_y = {},
      lualine_z = { "location" },
    },
  })
end

return {
  "nvim-lualine/lualine.nvim",
  dependencies = { "nvim-tree/nvim-web-devicons", "catppuccin" },
  config = function()
    setup()
    vim.api.nvim_create_autocmd("ColorScheme", {
      group = vim.api.nvim_create_augroup("f0_lualine_theme", { clear = true }),
      callback = setup,
    })
  end,
}
