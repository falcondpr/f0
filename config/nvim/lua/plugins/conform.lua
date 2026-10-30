return {
  "stevearc/conform.nvim",
  event = { "BufReadPre", "BufNewFile" },
  config = function()
    require("conform").setup({
      formatters_by_ft = {
        javascript     = { "biome-check" },
        javascriptreact = { "biome-check" },
        typescript     = { "biome-check" },
        typescriptreact = { "biome-check" },
        json           = { "biome-check" },
        jsonc          = { "biome-check" },
        css            = { "biome-check" },
      },
      format_on_save = {
        timeout_ms = 3000,
        lsp_format = "never",
      },
    })

    vim.keymap.set({ "n", "v" }, "<leader>cf", function()
      require("conform").format({ async = false })
    end, { desc = "Formatear con Biome" })
  end,
}
