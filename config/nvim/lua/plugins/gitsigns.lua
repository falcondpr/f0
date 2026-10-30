return {
  "lewis6991/gitsigns.nvim",
  event = { "BufReadPre", "BufNewFile" },
  config = function()
    require("gitsigns").setup({
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
