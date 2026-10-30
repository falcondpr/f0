return {
  "nvim-neo-tree/neo-tree.nvim",
  branch = "v3.x",
  dependencies = {
    "nvim-lua/plenary.nvim",
    "nvim-tree/nvim-web-devicons",
    "MunifTanjim/nui.nvim",
  },
  config = function()
    require("neo-tree").setup({
      close_if_last_window = true,

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
        width = 38,
      },
      filesystem = {
        window = {
          position = "right",
          width = 38,
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
      },
      git_status = {
        window = {
          position = "left",
          width = 34,
        },
      },
    })

    local function ide_layout()
      vim.cmd("Neotree show left git_status")
      vim.cmd("Neotree show right filesystem")
      -- devolver el foco al panel central (el código)
      for _, win in ipairs(vim.api.nvim_tabpage_list_wins(0)) do
        local buf = vim.api.nvim_win_get_buf(win)
        if vim.bo[buf].filetype ~= "neo-tree" then
          vim.api.nvim_set_current_win(win)
          break
        end
      end
    end

    vim.api.nvim_create_user_command("IDE", ide_layout, { desc = "Layout de 3 paneles" })

    local map = vim.keymap.set
    map("n", "<leader>e", "<cmd>Neotree toggle right filesystem<cr>",
      { desc = "Explorador de archivos (derecha)" })
    map("n", "<leader>gm", "<cmd>Neotree toggle left git_status<cr>",
      { desc = "Archivos modificados (izquierda)" })
    map("n", "<leader>ui", ide_layout, { desc = "Layout IDE (3 paneles)" })

    -- Al abrir `nvim .` sobre un directorio, montar el layout completo
    vim.api.nvim_create_autocmd("VimEnter", {
      callback = function()
        local arg = vim.fn.argv(0)
        if type(arg) == "string" and arg ~= "" and vim.fn.isdirectory(arg) == 1 then
          vim.schedule(ide_layout)
        end
      end,
    })
  end,
}
