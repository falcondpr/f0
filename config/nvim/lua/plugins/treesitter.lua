return {
  "nvim-treesitter/nvim-treesitter",
  branch = "main",
  build = ":TSUpdate",
  config = function()
    local parsers = {
      "typescript", "tsx", "javascript", "json",
      "lua", "markdown", "bash", "css",
    }

    require("nvim-treesitter").install(parsers)

    vim.api.nvim_create_autocmd("FileType", {
      pattern = {
        "typescript", "typescriptreact", "javascript", "javascriptreact",
        "json", "lua", "markdown", "sh", "bash", "css",
      },
      callback = function()
        pcall(vim.treesitter.start)
      end,
    })
  end,
}
