return {
  "lewis6991/gitsigns.nvim",
  event = { "BufReadPre", "BufNewFile" },
  config = function()
    -- Una barra fina en la columna de signos; el color dice qué cambió.
    local bar = { text = "▎" }
    local signs = {
      add = bar, change = bar, delete = bar, topdelete = bar, changedelete = bar, untracked = bar,
    }
    require("gitsigns").setup({
      signs = signs,
      signs_staged = signs,
      on_attach = function(bufnr)
        local gs = require("gitsigns")
        local function map(mode, l, r, desc)
          vim.keymap.set(mode, l, r, { buffer = bufnr, desc = desc })
        end

        map("n", "]c", function()
          if vim.wo.diff then return "]c" end
          vim.schedule(function() gs.nav_hunk("next") end)
          return "<Ignore>"
        end, "Siguiente cambio (hunk)")

        map("n", "[c", function()
          if vim.wo.diff then return "[c" end
          vim.schedule(function() gs.nav_hunk("prev") end)
          return "<Ignore>"
        end, "Anterior cambio (hunk)")

        map("n", "<leader>gp", gs.preview_hunk,         "Previsualizar cambio")
        map("n", "<leader>gr", gs.reset_hunk,           "Deshacer bloque de cambios")
        map("n", "<leader>gb", function() gs.blame_line({ full = true }) end, "git blame de la línea")
      end,
    })
  end,
}
