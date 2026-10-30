return {
  "neovim/nvim-lspconfig",
  event = { "BufReadPre", "BufNewFile" },
  dependencies = {
    "williamboman/mason.nvim",
    "williamboman/mason-lspconfig.nvim",
    "hrsh7th/cmp-nvim-lsp",
  },
  config = function()
    require("mason").setup()

    require("mason-lspconfig").setup({
      ensure_installed = { "ts_ls" },
    })

    local capabilities = require("cmp_nvim_lsp").default_capabilities()
    vim.lsp.config("ts_ls", {
      capabilities = capabilities,
    })
    vim.lsp.enable("ts_ls")

    vim.api.nvim_create_autocmd("LspAttach", {
      callback = function(ev)
        local client = vim.lsp.get_client_by_id(ev.data.client_id)
        if client then
          client.server_capabilities.documentFormattingProvider = false
          client.server_capabilities.documentRangeFormattingProvider = false
        end

        local function map(lhs, rhs, desc)
          vim.keymap.set("n", lhs, rhs, { buffer = ev.buf, desc = desc })
        end

        map("gd", vim.lsp.buf.definition, "Ir a definición")
        map("gD", vim.lsp.buf.declaration, "Ir a declaración")
        map("K", vim.lsp.buf.hover, "Hover (tipos)")
        map("gi", vim.lsp.buf.implementation, "Implementación")
        map("gr", vim.lsp.buf.references, "Referencias")
        map("<leader>rn", vim.lsp.buf.rename, "Renombrar")
        map("<leader>ca", vim.lsp.buf.code_action, "Code action")
        map("[d", function() vim.diagnostic.jump({ count = -1, float = true }) end, "Diagnóstico anterior")
        map("]d", function() vim.diagnostic.jump({ count = 1, float = true }) end, "Diagnóstico siguiente")
        map("<leader>ld", vim.diagnostic.open_float, "Ver diagnóstico")
      end,
    })
  end,
}
