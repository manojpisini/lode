local M = {}

M.defaults = {
  bin_path = "lode",
  enable_diagnostics = true,
  sign_column = true,
  check_on_save = true,
  scan_on_save = false,
  diagnostics_level = vim.diagnostic.severity.WARN,
}

M.options = {}

function M.setup(opts)
  M.options = vim.tbl_deep_extend("force", vim.deepcopy(M.defaults), opts or {})
  vim.g.lode_bin_path = M.options.bin_path
  if M.options.keybindings then
    M.setup_keymaps()
  end
end

function M.setup_keymaps()
  local map = vim.keymap.set
  local opts = { silent = true, noremap = true }
  map("n", "<leader>lc", "<cmd>LodeCheck<CR>", vim.tbl_extend("force", opts, { desc = "Lode check" }))
  map("n", "<leader>ls", "<cmd>LodeScan<CR>", vim.tbl_extend("force", opts, { desc = "Lode scan secrets" }))
  map("n", "<leader>li", "<cmd>LodeInit<CR>", vim.tbl_extend("force", opts, { desc = "Lode init project" }))
  map("n", "<leader>ly", "<cmd>LodeSync<CR>", vim.tbl_extend("force", opts, { desc = "Lode sync" }))
  map("n", "<leader>lz", "<cmd>LodeStatus<CR>", vim.tbl_extend("force", opts, { desc = "Lode status" }))
end

return M
