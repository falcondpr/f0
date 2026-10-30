-- Ctrl-h/j/k/l cruza entre ventanas de nvim y paneles de tmux con las mismas teclas
return {
  "christoomey/vim-tmux-navigator",
  cmd = { "TmuxNavigateLeft", "TmuxNavigateDown", "TmuxNavigateUp", "TmuxNavigateRight" },
  keys = {
    { "<C-h>", "<cmd>TmuxNavigateLeft<cr>", desc = "Panel izquierdo" },
    { "<C-j>", "<cmd>TmuxNavigateDown<cr>", desc = "Panel de abajo" },
    { "<C-k>", "<cmd>TmuxNavigateUp<cr>", desc = "Panel de arriba" },
    { "<C-l>", "<cmd>TmuxNavigateRight<cr>", desc = "Panel derecho" },
  },
}
