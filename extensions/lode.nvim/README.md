# lode.nvim

Neovim plugin for [LODE](https://github.com/anomalyco/lode) — a local Rust developer tool for convention checking, secret scanning, project scaffolding, and more.

## Requirements

- Neovim >= 0.9.0
- `lode` binary in your `$PATH` (or configured via `bin_path`)

## Installation

### [lazy.nvim](https://github.com/folke/lazy.nvim)

```lua
{
  "D:\\Manoj\\Project\\04_Languages\\Rust\\lode\\extensions\\lode.nvim",
  -- or from a remote source:
  -- dir = "~/path/to/lode/extensions/lode.nvim",
  opts = {
    bin_path = "lode",
    enable_diagnostics = true,
    sign_column = true,
    check_on_save = true,
  },
}
```

### [packer.nvim](https://github.com/wbthomason/packer.nvim)

```lua
use {
  "D:\\Manoj\\Project\\04_Languages\\Rust\\lode\\extensions\\lode.nvim",
  config = function()
    require("lode").setup({
      bin_path = "lode",
      enable_diagnostics = true,
    })
  end,
}
```

### [vim-plug](https://github.com/junegunn/vim-plug)

```vim
Plug 'D:\Manoj\Project\04_Languages\Rust\lode\extensions\lode.nvim'

" In your init.vim / lua:
" lua require('lode').setup({ bin_path = 'lode' })
```

## Commands

| Command | Description |
|---|---|
| `:LodeCheck [path]` | Check file or project for naming convention violations |
| `:LodeScan [path]` | Scan project for secrets |
| `:LodeInit <name>` | Initialize a new LODE project |
| `:LodeSync` | Sync templates and config |
| `:LodeStatus` | Show project health summary in a floating window |

## Configuration

```lua
require("lode").setup({
  bin_path = "lode",           -- path to lode binary
  enable_diagnostics = true,   -- enable diagnostics integration
  sign_column = true,          -- show signs for violations
  check_on_save = true,        -- auto-run lode check on save
  scan_on_save = false,        -- auto-run lode scan on save
  diagnostics_level = vim.diagnostic.severity.WARN,
})
```

## API

```lua
local lode = require("lode")
lode.commands.check({ args = "path/to/file" })
lode.commands.scan({})
lode.diagnostics.run_check()
lode.diagnostics.run_scan()
```
