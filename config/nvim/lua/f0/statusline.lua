-- Una sola barra, sin plugin, en el tono de los paneles y con el texto apagado: a la
-- izquierda la ruta relativa y si hay cambios sin guardar, a la derecha los
-- diagnósticos y `línea:columna`. La rama y la carpeta ya están en la barra de tmux.
-- El modo se ve en la forma del cursor; acá aparece solo fuera de normal, en el acento.
local M = {}

local modes = {
  i = "INSERT",
  v = "VISUAL",
  V = "V-LINE",
  ["\22"] = "V-BLOCK",
  s = "SELECT",
  S = "S-LINE",
  ["\19"] = "S-BLOCK",
  R = "REPLACE",
  c = "COMMAND",
  t = "TERMINAL",
}

local severities = {
  { vim.diagnostic.severity.ERROR, "E" },
  { vim.diagnostic.severity.WARN, "W" },
  { vim.diagnostic.severity.INFO, "I" },
  { vim.diagnostic.severity.HINT, "H" },
}

local function mode()
  local name = modes[vim.api.nvim_get_mode().mode:sub(1, 1)]
  return name and ("%#F0StatusMode#" .. name .. "%*  ") or ""
end

local function path()
  if vim.bo.buftype ~= "" then
    return ""
  end
  local name = vim.api.nvim_buf_get_name(0)
  if name == "" then
    return ""
  end
  local relative = vim.fn.fnamemodify(name, ":~:."):gsub("%%", "%%%%")
  return relative .. (vim.bo.modified and " [+]" or "")
end

local function diagnostics()
  local counts = vim.diagnostic.count(0)
  local parts = {}
  for _, entry in ipairs(severities) do
    local count = counts[entry[1]]
    if count and count > 0 then
      table.insert(parts, entry[2] .. count)
    end
  end
  return table.concat(parts, " ")
end

function M.render()
  local right = diagnostics()
  if vim.bo.buftype == "" then
    right = right .. (right ~= "" and "  " or "") .. "%l:%c"
  end
  return " " .. mode() .. path() .. "%=" .. right .. " "
end

vim.o.statusline = "%{%v:lua.require'f0.statusline'.render()%}"

vim.api.nvim_create_autocmd({ "ModeChanged", "DiagnosticChanged" }, {
  group = vim.api.nvim_create_augroup("f0_statusline", { clear = true }),
  callback = function()
    vim.cmd.redrawstatus()
  end,
})

return M
