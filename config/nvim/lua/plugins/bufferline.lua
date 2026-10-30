return {
  "akinsho/bufferline.nvim",
  event = "VeryLazy",
  dependencies = { "nvim-tree/nvim-web-devicons" },
  config = function()
    require("bufferline").setup({
      options = {
        diagnostics = "nvim_lsp",
        offsets = {
          {
            filetype = "neo-tree",
            text = "Explorer",
            separator = true,
          },
        },
      },
    })

    local map = vim.keymap.set
    map("n", "<S-l>", "<cmd>BufferLineCycleNext<cr>", { desc = "Buffer siguiente" })
    map("n", "<S-h>", "<cmd>BufferLineCyclePrev<cr>", { desc = "Buffer anterior" })
    map("n", "<leader>bd", function()
      local buf = vim.api.nvim_get_current_buf()
      local force = false
      -- Decidir antes de cambiar de buffer: si `bdelete` falla, ya no estaríamos en él.
      if vim.bo[buf].modified then
        local name = vim.fn.fnamemodify(vim.api.nvim_buf_get_name(buf), ":t")
        local choice = vim.fn.confirm(
          ("¿Guardar los cambios de %s?"):format(name ~= "" and name or "[sin nombre]"),
          "&Guardar\n&Descartar\n&Cancelar",
          3
        )
        if choice == 1 then
          local ok, err = pcall(vim.cmd, "write")
          if not ok then
            vim.notify(err, vim.log.levels.ERROR)
            return
          end
        elseif choice == 2 then
          force = true
        else
          return
        end
      end
      vim.cmd("bprevious")
      if vim.api.nvim_get_current_buf() == buf then
        vim.cmd("enew")
      end
      vim.cmd((force and "bdelete! " or "bdelete ") .. buf)
    end, { desc = "Cerrar buffer" })
  end,
}
