-- 'background' sigue el modo elegido en el launcher de f0 (`t`), o si no hay uno, la
-- apariencia de macOS, como tmux (config/tmux/appearance.sh). nvim la toma de la
-- terminal al arrancar, pero no todas las terminales avisan cuando cambia (WezTerm no),
-- así que se consulta al volver el foco y cada pocos segundos. Solo se escribe cuando
-- el modo cambió: un `:set background` a mano se respeta hasta el próximo cambio.
local config_dir = vim.env.F0_CONFIG_DIR
  or vim.fs.joinpath(vim.env.XDG_CONFIG_HOME or vim.fs.joinpath(vim.env.HOME, ".config"), "f0")
local picked_file = vim.fs.joinpath(config_dir, "appearance")
local has_defaults = vim.fn.executable("/usr/bin/defaults") == 1

local last

local function apply(mode)
  if mode == last then
    return
  end
  last = mode
  vim.schedule(function()
    if vim.o.background ~= mode then
      vim.o.background = mode
    end
  end)
end

local function picked()
  local file = io.open(picked_file, "r")
  if not file then
    return nil
  end
  local word = vim.trim(file:read("*a") or "")
  file:close()
  return (word == "light" or word == "dark") and word or nil
end

local function check()
  local mode = picked()
  if mode then
    apply(mode)
  elseif has_defaults then
    vim.system({ "/usr/bin/defaults", "read", "-g", "AppleInterfaceStyle" }, { text = true }, function(result)
      apply(vim.trim(result.stdout or "") == "Dark" and "dark" or "light")
    end)
  end
end

vim.api.nvim_create_autocmd({ "VimEnter", "FocusGained" }, {
  group = vim.api.nvim_create_augroup("f0_appearance", { clear = true }),
  callback = check,
})

local timer = vim.uv.new_timer()
if timer then
  timer:start(5000, 5000, vim.schedule_wrap(check))
end
