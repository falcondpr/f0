-- Ancho de los paneles laterales (neo-tree y el file panel de diffview). El editor
-- comparte su pane de tmux con el agente y tiene un panel a cada lado: con anchos
-- fijos el código quedaba tan angosto como cada panel. Una parte del ancho, con
-- tope arriba y abajo, deja al código la mayor parte y los paneles siguen legibles.
local M = {}

local share = 0.18
local min_width = 22
local max_width = 30

local side_filetypes = { ["neo-tree"] = true, DiffviewFiles = true }

function M.side_width()
  return math.max(min_width, math.min(max_width, math.floor(vim.o.columns * share)))
end

-- neo-tree y diffview calculan el ancho solo al abrir el panel: al redimensionar
-- f0.app o el pane de tmux quedaban con el ancho anterior. Nvim además saca o agrega
-- las columnas en la ventana de más a la derecha, y los dos lados de un diff quedaban
-- desparejos: `horizontal wincmd =` reparte el resto y respeta los paneles (`winfixwidth`).
local group = vim.api.nvim_create_augroup("f0_panels", { clear = true })
vim.api.nvim_create_autocmd("VimResized", {
  group = group,
  callback = function()
    local width = M.side_width()
    for _, win in ipairs(vim.api.nvim_tabpage_list_wins(0)) do
      local buf = vim.api.nvim_win_get_buf(win)
      if side_filetypes[vim.bo[buf].filetype] and vim.api.nvim_win_get_config(win).relative == "" then
        vim.api.nvim_win_set_width(win, width)
      end
    end
    vim.cmd("horizontal wincmd =")
  end,
})

return M
