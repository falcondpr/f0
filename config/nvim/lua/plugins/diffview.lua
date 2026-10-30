-- f0 exporta F0_BASE en cada sesion (la rama base del proyecto)
local function base_branch()
  return vim.env.F0_BASE or "main"
end

return {
  "sindrets/diffview.nvim",
  cmd = { "DiffviewOpen", "DiffviewFileHistory" },
  keys = {
    { "<leader>gd", "<cmd>DiffviewOpen<cr>", desc = "Diff de cambios sin commitear" },
    {
      "<leader>gD",
      function()
        vim.cmd("DiffviewOpen origin/" .. base_branch() .. "...HEAD")
      end,
      desc = "Diff de la rama contra la base",
    },
    { "<leader>gh", "<cmd>DiffviewFileHistory %<cr>", desc = "Historial del archivo" },
    { "<leader>gq", "<cmd>DiffviewClose<cr>", desc = "Cerrar diff" },
  },
  opts = {
    enhanced_diff_hl = true,
    view = {
      default = { layout = "diff2_horizontal" },
    },
    file_panel = {
      listing_style = "tree",
      win_config = { position = "left", width = 34 },
    },
  },
}
