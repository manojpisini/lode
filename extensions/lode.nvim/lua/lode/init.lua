local M = {}

M.config = require("lode.config")
M.diagnostics = require("lode.diagnostics")
M.commands = require("lode.commands")

function M.setup(opts)
  M.config.setup(opts or {})

  if M.config.options.enable_diagnostics then
    M.diagnostics.setup_autocmds()
  end

  if M.config.options.sign_column then
    M.diagnostics.setup_signs()
  end

  M.commands.setup()

  vim.api.nvim_create_user_command("LodeCheck", function(args)
    M.commands.check(args)
  end, { nargs = "?", complete = "file", desc = "Run lode check on a file or project" })

  vim.api.nvim_create_user_command("LodeScan", function(args)
    M.commands.scan(args)
  end, { nargs = "?", desc = "Scan project for secrets" })

  vim.api.nvim_create_user_command("LodeInit", function(args)
    M.commands.init(args)
  end, { nargs = 1, complete = "file", desc = "Initialize a new lode project" })

  vim.api.nvim_create_user_command("LodeSync", function(args)
    M.commands.sync(args)
  end, { bang = true, desc = "Sync lode templates and config" })

  vim.api.nvim_create_user_command("LodeStatus", function()
    M.commands.status()
  end, { desc = "Show project health summary" })
end

return M
