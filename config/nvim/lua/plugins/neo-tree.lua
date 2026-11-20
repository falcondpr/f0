return {
  "nvim-neo-tree/neo-tree.nvim",
  branch = "v3.x",
  dependencies = {
    "nvim-lua/plenary.nvim",
    "MunifTanjim/nui.nvim",
  },
  config = function()
    local side_width = require("f0.panels").side_width

    require("neo-tree").setup({
      close_if_last_window = true,
      -- La raíz repetía la ruta (`GIT STATUS for ~/…`), que ya muestran el título del
      -- panel y la barra de tmux.
      hide_root_node = true,

      default_component_configs = {
        -- Una lista simple: sin íconos de archivo ni guías, las carpetas con una
        -- flecha. Los archivos llevan un espacio en el lugar del ícono para quedar
        -- alineados con el nombre de las carpetas.
        indent = {
          with_markers = false,
        },
        icon = {
          folder_closed = "›",
          folder_open = "⌄",
          folder_empty = "›",
          folder_empty_open = "⌄",
          default = " ",
          provider = function() end,
        },
        -- Una sola marca por archivo: la letra del cambio en su color. Los símbolos de
        -- stage repetían lo mismo con otro ícono; staged vs. unstaged se ve en diffview.
        git_status = {
          symbols = {
            added = "A",
            deleted = "D",
            modified = "M",
            renamed = "R",
            untracked = "?",
            ignored = "",
            unstaged = "",
            staged = "",
            conflict = "!",
          },
        },
      },

      -- El panel git_status pide `git status --ignored=traditional -uall`, que en un
      -- monorepo enumera todo node_modules (~230k entradas, ~1.8 s) para luego
      -- descartarlo. Solo esa llamada usa `-uall`, así que la reconocemos por ahí
      -- y le quitamos `--ignored`. El panel de archivos usa `--ignored=traditional`
      -- normal (async) y conserva su `hide_gitignored`.
      event_handlers = {
        {
          event = "before_git_status",
          handler = function(args)
            local a = args.status_args
            if not a then
              return
            end
            local is_git_status_panel = false
            for _, v in ipairs(a) do
              if v == "--untracked-files=all" then
                is_git_status_panel = true
                break
              end
            end
            if not is_git_status_panel then
              return
            end
            for i, v in ipairs(a) do
              if type(v) == "string" and v:match("^%-%-ignored=") then
                a[i] = "--ignored=no"
              end
            end
          end,
        },
      },

      window = {
        width = side_width,
      },
      -- Explorer y cambios comparten el lado izquierdo: abrir uno reemplaza al otro.
      -- Con un panel a cada lado, y el agente al costado, el código y los paneles
      -- quedaban angostos a la vez.
      filesystem = {
        window = {
          position = "left",
          width = side_width,
        },
        filtered_items = {
          visible = true,
          hide_dotfiles = false,
          hide_gitignored = true,
        },
        follow_current_file = {
          enabled = true,
        },
        use_libuv_file_watcher = true,
        -- El hijack abre el explorer 10 ms después del arranque y cierra los demás
        -- paneles, también el de cambios que abre `nvim .`.
        hijack_netrw_behavior = "disabled",
      },
      git_status = {
        window = {
          position = "left",
          width = side_width,
        },
      },
    })

    local function ide_layout()
      -- El foco va al panel central (el código) antes de abrir los paneles:
      -- `Neotree show` devuelve el foco, cuando termina, a la ventana que estaba activa.
      for _, win in ipairs(vim.api.nvim_tabpage_list_wins(0)) do
        local buf = vim.api.nvim_win_get_buf(win)
        if vim.bo[buf].filetype ~= "neo-tree" then
          vim.api.nvim_set_current_win(win)
          break
        end
      end
      vim.cmd("Neotree show left git_status")
    end

    -- El panel git_status no escucha el filesystem, y el watcher del explorer solo
    -- cubre carpetas expandidas: lo que el agente escribe en carpetas colapsadas no
    -- aparecía. Un watcher recursivo (FSEvents en macOS) sobre la raíz del repo
    -- refresca solo git_status, con debounce para que una ráfaga de escrituras
    -- corra un único `git status`.
    local git_status_watcher, git_status_timer

    local function git_status_panel_visible()
      for _, win in ipairs(vim.api.nvim_tabpage_list_wins(0)) do
        if vim.b[vim.api.nvim_win_get_buf(win)].neo_tree_source == "git_status" then
          return true
        end
      end
      return false
    end

    local function refresh_git_status()
      if git_status_panel_visible() then
        require("neo-tree.sources.manager").refresh("git_status")
      end
    end

    -- .git ya lo vigila neo-tree, y además `git status` reescribe .git/index:
    -- escucharlo dispararía un refresh tras otro.
    local function is_ignored_change(filename)
      if not filename then
        return false
      end
      local path = "/" .. filename .. "/"
      return path:find("/%.git/") ~= nil or path:find("/node_modules/") ~= nil
    end

    local function stop_git_status_watcher()
      for _, handle in ipairs({ git_status_watcher, git_status_timer }) do
        if not handle:is_closing() then
          handle:close()
        end
      end
      git_status_watcher, git_status_timer = nil, nil
    end

    local function start_git_status_watcher()
      if git_status_watcher then
        stop_git_status_watcher()
      end
      local root = vim.fs.root(vim.fn.getcwd(), ".git")
      if not root then
        return
      end
      local timer = vim.uv.new_timer()
      local watcher = vim.uv.new_fs_event()
      if not timer or not watcher then
        return
      end
      git_status_timer, git_status_watcher = timer, watcher
      watcher:start(root, { recursive = true }, function(err, filename)
        if err or is_ignored_change(filename) then
          return
        end
        timer:stop()
        timer:start(200, 0, vim.schedule_wrap(refresh_git_status))
      end)
    end

    local group = vim.api.nvim_create_augroup("f0_git_status_watcher", { clear = true })
    vim.api.nvim_create_autocmd("DirChanged", { group = group, callback = start_git_status_watcher })
    vim.api.nvim_create_autocmd("VimLeavePre", {
      group = group,
      callback = function()
        if git_status_watcher then
          stop_git_status_watcher()
        end
      end,
    })
    start_git_status_watcher()

    vim.api.nvim_create_user_command("IDE", ide_layout, { desc = "Layout con el panel de cambios" })

    local map = vim.keymap.set
    map("n", "<leader>e", "<cmd>Neotree toggle left filesystem<cr>",
      { desc = "Explorador de archivos (izquierda)" })
    map("n", "<leader>gm", "<cmd>Neotree toggle left git_status<cr>",
      { desc = "Archivos modificados (izquierda)" })
    map("n", "<leader>ui", ide_layout, { desc = "Layout IDE (panel de cambios)" })

    -- `nvim .` deja abierto el buffer del directorio: va uno vacío en su lugar, con los
    -- atajos de f0/empty.lua, y al lado el panel de cambios.
    vim.api.nvim_create_autocmd("VimEnter", {
      callback = function()
        local arg = vim.fn.argv(0)
        if type(arg) == "string" and arg ~= "" and vim.fn.isdirectory(arg) == 1 then
          vim.schedule(function()
            local directory = vim.api.nvim_get_current_buf()
            vim.cmd("enew")
            pcall(vim.api.nvim_buf_delete, directory, { force = true })
            ide_layout()
          end)
        end
      end,
    })
  end,
}
