return {
  "nvim-telescope/telescope.nvim",
  branch = "master",
  dependencies = { "nvim-lua/plenary.nvim" },
  config = function()
    local telescope = require("telescope")
    telescope.setup({
      defaults = {
        mappings = {
          i = {
            ["<C-n>"] = "move_selection_next",
            ["<C-p>"] = "move_selection_previous",
          },
        },
      },
    })

    local builtin = require("telescope.builtin")
    local map = vim.keymap.set
    map("n", "<leader>ff", builtin.find_files,  { desc = "Buscar archivos" })
    map("n", "<leader>fg", builtin.live_grep,   { desc = "Buscar texto (grep)" })
    map("n", "<leader>fb", builtin.buffers,     { desc = "Buffers abiertos" })
    map("n", "<leader>fr", builtin.oldfiles,    { desc = "Archivos recientes" })
    map("n", "<leader>fh", builtin.help_tags,   { desc = "Ayuda de nvim" })
    map("n", "<leader>gs", builtin.git_status,  { desc = "Archivos modificados (git status)" })
    map("n", "<leader>gc", builtin.git_commits, { desc = "Commits (git log)" })
  end,
}
