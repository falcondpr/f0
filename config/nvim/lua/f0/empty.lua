-- Sin archivo abierto, el centro mostraba un `[No Name]` con números de línea. En su
-- lugar queda vacío, con una línea de atajos centrada.
local ns = vim.api.nvim_create_namespace("f0_empty")

local hints = {
  { "space ff", "find file" },
  { "space fr", "recent" },
  { "space e", "explorer" },
  { "space gd", "diff" },
}

vim.api.nvim_set_hl(0, "F0EmptyKey", { link = "Special", default = true })
vim.api.nvim_set_hl(0, "F0EmptyText", { link = "Comment", default = true })

local function is_empty(buf)
  return vim.api.nvim_buf_get_name(buf) == ""
    and vim.bo[buf].buftype == ""
    and vim.bo[buf].filetype == ""
    and not vim.bo[buf].modified
    and vim.api.nvim_buf_line_count(buf) == 1
    and vim.api.nvim_buf_get_lines(buf, 0, 1, false)[1] == ""
end

local gap = "    "

-- Los atajos que entran en el ancho de la ventana, en orden: entre los dos paneles de
-- neo-tree el centro puede quedar angosto.
local function render(win, buf)
  local available = vim.api.nvim_win_get_width(win) - 4
  local chunks, width = {}, 0
  for _, hint in ipairs(hints) do
    local key, text = hint[1], " " .. hint[2]
    local extra = (width > 0 and #gap or 0) + #key + #text
    if width + extra > available then
      break
    end
    if width > 0 then
      table.insert(chunks, { gap })
    end
    table.insert(chunks, { key, "F0EmptyKey" })
    table.insert(chunks, { text, "F0EmptyText" })
    width = width + extra
  end
  local pad = math.max(0, math.floor((vim.api.nvim_win_get_width(win) - width) / 2))
  table.insert(chunks, 1, { string.rep(" ", pad) })
  local lines = {}
  for _ = 1, math.max(0, math.floor(vim.api.nvim_win_get_height(win) / 2) - 1) do
    table.insert(lines, { { "" } })
  end
  table.insert(lines, chunks)
  vim.api.nvim_buf_clear_namespace(buf, ns, 0, -1)
  vim.api.nvim_buf_set_extmark(buf, ns, 0, 0, { virt_lines = lines })
end

-- Solo los valores locales (`[0]`, como `:setlocal`): `vim.wo[win]` cambiaría también
-- el global. Al abrir un archivo en la ventana se le devuelven los números, por si
-- hereda los valores del buffer vacío.
local function update(win)
  if not vim.api.nvim_win_is_valid(win) or vim.api.nvim_win_get_config(win).relative ~= "" then
    return
  end
  local buf = vim.api.nvim_win_get_buf(win)
  if is_empty(buf) then
    vim.w[win].f0_empty = true
    local wo = vim.wo[win][0]
    wo.number = false
    wo.relativenumber = false
    wo.signcolumn = "no"
    render(win, buf)
  else
    vim.api.nvim_buf_clear_namespace(buf, ns, 0, -1)
    if vim.w[win].f0_empty then
      vim.w[win].f0_empty = nil
      local wo = vim.wo[win][0]
      wo.number = vim.go.number
      wo.relativenumber = vim.go.relativenumber
      wo.signcolumn = vim.go.signcolumn
    end
  end
end

local group = vim.api.nvim_create_augroup("f0_empty", { clear = true })
vim.api.nvim_create_autocmd({ "VimEnter", "BufWinEnter", "BufEnter", "TextChanged", "TextChangedI", "BufModifiedSet" }, {
  group = group,
  callback = function()
    update(vim.api.nvim_get_current_win())
  end,
})
vim.api.nvim_create_autocmd({ "VimResized", "WinResized" }, {
  group = group,
  callback = function()
    for _, win in ipairs(vim.api.nvim_tabpage_list_wins(0)) do
      update(win)
    end
  end,
})
