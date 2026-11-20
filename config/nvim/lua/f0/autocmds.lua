vim.api.nvim_create_autocmd({ "FocusGained", "BufEnter", "CursorHold" }, {
  callback = function()
    if vim.fn.mode() ~= "c" then
      vim.cmd("checktime")
    end
  end,
})

local group = vim.api.nvim_create_augroup("f0_ui", { clear = true })

-- Una guía por nivel de indentación: `▏` seguido de `shiftwidth - 1` espacios.
local function indent_guides()
  local listchars = vim.opt_global.listchars:get()
  listchars.leadmultispace = "▏" .. string.rep(" ", math.max(vim.fn.shiftwidth() - 1, 0))
  vim.opt_local.listchars = listchars
end

vim.api.nvim_create_autocmd({ "BufWinEnter", "FileType" }, { group = group, callback = indent_guides })
vim.api.nvim_create_autocmd("OptionSet", {
  group = group,
  pattern = { "shiftwidth", "tabstop" },
  callback = indent_guides,
})

-- La línea del cursor solo en la ventana con foco y en buffers de archivos: neo-tree
-- maneja la suya, que marca el archivo actual aunque el foco esté en el editor.
local function is_file_window()
  return vim.bo.buftype == "" and not vim.w.f0_empty
end

vim.api.nvim_create_autocmd({ "VimEnter", "WinEnter", "BufWinEnter" }, {
  group = group,
  callback = function()
    if is_file_window() then
      vim.wo[0][0].cursorline = true
    end
  end,
})
vim.api.nvim_create_autocmd("WinLeave", {
  group = group,
  callback = function()
    if vim.bo.buftype == "" then
      vim.wo[0][0].cursorline = false
    end
  end,
})
