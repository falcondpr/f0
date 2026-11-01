-- 'background' sigue la apariencia de macOS, como tmux (config/tmux/appearance.sh).
-- nvim la toma de la terminal al arrancar, pero no todas las terminales avisan cuando
-- cambia (WezTerm no), así que se consulta al volver el foco y cada pocos segundos.
-- Solo se escribe cuando la apariencia cambió: un `:set background` a mano se respeta
-- hasta el próximo cambio.
if vim.fn.executable("/usr/bin/defaults") == 0 then
  return
end

local last

local function check()
  vim.system({ "/usr/bin/defaults", "read", "-g", "AppleInterfaceStyle" }, { text = true }, function(result)
    local mode = vim.trim(result.stdout or "") == "Dark" and "dark" or "light"
    if mode == last then
      return
    end
    last = mode
    vim.schedule(function()
      if vim.o.background ~= mode then
        vim.o.background = mode
      end
    end)
  end)
end

vim.api.nvim_create_autocmd({ "VimEnter", "FocusGained" }, {
  group = vim.api.nvim_create_augroup("f0_appearance", { clear = true }),
  callback = check,
})

local timer = vim.uv.new_timer()
if timer then
  timer:start(5000, 5000, vim.schedule_wrap(check))
end
