local M = {}

local ns = vim.api.nvim_create_namespace("lode")

function M.setup_autocmds()
  local group = vim.api.nvim_create_augroup("lode_diagnostics", { clear = true })
  vim.api.nvim_create_autocmd("BufWritePost", {
    group = group,
    pattern = "*",
    callback = function()
      local conf = require("lode.config").options
      if conf.check_on_save then
        M.run_check()
      end
      if conf.scan_on_save then
        M.run_scan()
      end
    end,
  })
end

function M.setup_signs()
  vim.fn.sign_define("LodeViolation", { text = "⚠", texthl = "DiagnosticSignWarn" })
  vim.fn.sign_define("LodeSecret", { text = "🔑", texthl = "DiagnosticSignError" })
end

function M.run_check()
  local bufnr = vim.api.nvim_get_current_buf()
  local path = vim.api.nvim_buf_get_name(bufnr)
  if path == "" then
    return
  end
  local conf = require("lode.config").options
  local bin = conf.bin_path
  local cmd = { bin, "check", "--json", path }
  local job_id = vim.fn.jobstart(cmd, {
    stdout_buffered = true,
    stderr_buffered = true,
    on_stdout = function(_, data)
      if not data or #data == 0 then
        return
      end
      local ok, report = pcall(vim.json.decode, table.concat(data, "\n"))
      if not ok or not report then
        return
      end
      M.set_convention_diagnostics(bufnr, report)
    end,
    on_stderr = function(_, data)
      if data and #data > 0 then
        vim.schedule(function()
          vim.notify("[lode] check error: " .. table.concat(data, "\n"), vim.log.levels.ERROR)
        end)
      end
    end,
  })
  if job_id <= 0 then
    vim.notify("[lode] failed to start check: " .. bin .. " not found", vim.log.levels.ERROR)
  end
end

function M.run_scan()
  local bufnr = vim.api.nvim_get_current_buf()
  local path = vim.fn.expand("%:p:h")
  if path == "" then
    return
  end
  local conf = require("lode.config").options
  local bin = conf.bin_path
  local cmd = { bin, "scan", "secrets", "--json", path }
  local job_id = vim.fn.jobstart(cmd, {
    stdout_buffered = true,
    stderr_buffered = true,
    on_stdout = function(_, data)
      if not data or #data == 0 then
        return
      end
      local ok, report = pcall(vim.json.decode, table.concat(data, "\n"))
      if not ok or not report then
        return
      end
      M.set_secret_diagnostics(bufnr, report)
    end,
    on_stderr = function(_, data)
      if data and #data > 0 then
        vim.schedule(function()
          vim.notify("[lode] scan error: " .. table.concat(data, "\n"), vim.log.levels.ERROR)
        end)
      end
    end,
  })
  if job_id <= 0 then
    vim.notify("[lode] failed to start scan: " .. bin .. " not found", vim.log.levels.ERROR)
  end
end

function M.set_convention_diagnostics(bufnr, report)
  if not report or not report.violations then
    return
  end
  local diagnostics = {}
  local buf_path = vim.api.nvim_buf_get_name(bufnr)
  for _, v in ipairs(report.violations) do
    if v.path == buf_path or vim.fn.fnamemodify(v.path, ":p") == buf_path then
      table.insert(diagnostics, {
        lnum = 0,
        col = 0,
        severity = require("lode.config").options.diagnostics_level,
        message = "expected name: " .. v.expected_name,
        source = "lode-convention",
      })
    end
  end
  vim.diagnostic.set(ns, bufnr, diagnostics)
end

function M.set_secret_diagnostics(bufnr, report)
  if not report or not report.findings then
    return
  end
  local diagnostics = {}
  local buf_path = vim.api.nvim_buf_get_name(bufnr)
  for _, f in ipairs(report.findings) do
    if f.path == buf_path or vim.fn.fnamemodify(f.path, ":p") == buf_path then
      table.insert(diagnostics, {
        lnum = math.max(0, f.line - 1),
        col = 0,
        severity = vim.diagnostic.severity.ERROR,
        message = "secret found: " .. f.kind,
        source = "lode-secrets",
      })
    end
  end
  vim.diagnostic.set(ns, bufnr, diagnostics)
end

function M.clear(bufnr)
  vim.diagnostic.reset(ns, bufnr)
end

return M
