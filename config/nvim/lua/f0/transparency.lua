local function make_transparent()
  local groups = {
    "Normal", "NormalNC", "NormalFloat", "FloatBorder",
    "SignColumn", "LineNr", "EndOfBuffer",
    "TelescopeNormal", "TelescopeBorder",
    "TelescopePromptNormal", "TelescopePromptBorder",
    "TelescopeResultsNormal", "TelescopePreviewNormal",
    "NeoTreeNormal", "NeoTreeNormalNC", "NeoTreeEndOfBuffer",
  }
  for _, group in ipairs(groups) do
    -- nvim_set_hl reemplaza el grupo entero: hay que conservar el fg. Sin él, plugins
    -- como diffview calculan sus colores desde Normal y caen en blanco.
    local hl = vim.api.nvim_get_hl(0, { name = group, link = false })
    hl.bg = nil
    hl.ctermbg = nil
    vim.api.nvim_set_hl(0, group, hl)
  end
end

vim.api.nvim_create_autocmd("ColorScheme", {
  callback = make_transparent,
})

make_transparent()
