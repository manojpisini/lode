local M = {}

function M.setup()
end

function M.check(args)
  local conf = require("lode.config").options
  local bin = conf.bin_path
  local path = args.args or vim.api.nvim_buf_get_name(0)
  if path == "" then
    path = "."
  end
  local cmd = { bin, "check", "--json", path }
  vim.notify("[lode] checking " .. vim.fn.fnamemodify(path, ":."), vim.log.levels.INFO)
  local output = vim.fn.system(cmd)
  if vim.v.shell_error ~= 0 then
    local ok, report = pcall(vim.json.decode, output)
    if ok and report and report.violations and #report.violations > 0 then
      M.show_violations(report)
    else
      vim.notify("[lode] check failed", vim.log.levels.ERROR)
    end
    return
  end
  local ok, report = pcall(vim.json.decode, output)
  if ok and report then
    if report.violations and #report.violations == 0 then
      vim.notify("[lode] convention ok: checked " .. (report.checked or 0) .. " files", vim.log.levels.INFO)
    else
      M.show_violations(report)
    end
  else
    vim.notify("[lode] " .. output, vim.log.levels.INFO)
  end
end

function M.show_violations(report)
  local lines = {}
  table.insert(lines, "Convention Violations")
  table.insert(lines, string.rep("=", 50))
  for _, v in ipairs(report.violations) do
    table.insert(lines, v.path .. "  =>  " .. v.expected_name)
  end
  if report.renamed and #report.renamed > 0 then
    table.insert(lines, "")
    table.insert(lines, "Renamed")
    table.insert(lines, string.rep("=", 50))
    for _, r in ipairs(report.renamed) do
      table.insert(lines, r[1] .. "  ->  " .. r[2])
    end
  end
  vim.lsp.util.open_floating_preview(lines, "plain", { border = "rounded", title = " Lode Check " })
end

function M.scan(args)
  local conf = require("lode.config").options
  local bin = conf.bin_path
  local path = args.args or vim.fn.expand("%:p:h")
  if path == "" then
    path = "."
  end
  local cmd = { bin, "scan", "secrets", "--json", path }
  vim.notify("[lode] scanning for secrets in " .. vim.fn.fnamemodify(path, ":."), vim.log.levels.INFO)
  local output = vim.fn.system(cmd)
  if vim.v.shell_error ~= 0 then
    local ok, report = pcall(vim.json.decode, output)
    if ok and report and report.findings and #report.findings > 0 then
      M.show_secret_findings(report)
    else
      vim.notify("[lode] scan failed", vim.log.levels.ERROR)
    end
    return
  end
  local ok, report = pcall(vim.json.decode, output)
  if ok and report then
    if report.findings and #report.findings == 0 then
      vim.notify("[lode] no secrets found", vim.log.levels.INFO)
    else
      M.show_secret_findings(report)
    end
  else
    vim.notify("[lode] " .. output, vim.log.levels.INFO)
  end
end

function M.show_secret_findings(report)
  local lines = {}
  table.insert(lines, "Secret Scan Results")
  table.insert(lines, string.rep("=", 50))
  table.insert(lines, "Checked files: " .. (report.checked_files or 0))
  table.insert(lines, "")
  for _, f in ipairs(report.findings) do
    table.insert(lines, f.path .. ":" .. f.line .. "  " .. f.kind)
  end
  vim.lsp.util.open_floating_preview(lines, "plain", { border = "rounded", title = " Lode Scan " })
end

function M.init(args)
  local conf = require("lode.config").options
  local bin = conf.bin_path
  local name = args.args
  if not name or name == "" then
    vim.notify("[lode] usage: LodeInit <project-name>", vim.log.levels.ERROR)
    return
  end
  local cmd = { bin, "init", name }
  vim.notify("[lode] initializing project: " .. name, vim.log.levels.INFO)
  local output = vim.fn.system(cmd)
  if vim.v.shell_error ~= 0 then
    vim.notify("[lode] init failed: " .. output, vim.log.levels.ERROR)
  else
    vim.notify("[lode] " .. output:gsub("\n", " "), vim.log.levels.INFO)
  end
end

function M.sync(args)
  local conf = require("lode.config").options
  local bin = conf.bin_path
  local cmd = { bin, "sync" }
  if args.bang then
    table.insert(cmd, "--force")
  end
  vim.notify("[lode] syncing templates and config", vim.log.levels.INFO)
  local output = vim.fn.system(cmd)
  if vim.v.shell_error ~= 0 then
    vim.notify("[lode] sync failed: " .. output, vim.log.levels.ERROR)
  else
    vim.notify("[lode] " .. output:gsub("\n", " "), vim.log.levels.INFO)
  end
end

function M.status()
  local conf = require("lode.config").options
  local bin = conf.bin_path
  local cmd = { bin, "health" }
  local output = vim.fn.system(cmd)
  if vim.v.shell_error ~= 0 then
    vim.notify("[lode] status check failed", vim.log.levels.ERROR)
    return
  end
  local lines = vim.split(output:gsub("\r\n", "\n"), "\n", { plain = true })
  vim.lsp.util.open_floating_preview(lines, "plain", { border = "rounded", title = " Lode Status " })
end

return M
