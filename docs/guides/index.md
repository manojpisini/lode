# Guides

## Getting Started

### Installation

**Prerequisites**:
- Rust toolchain: `stable-x86_64-pc-windows-gnu` (default)
- Windows: MSYS2 ucrt64 in PATH (`C:\msys64\ucrt64\bin`)
- Alternative: `rustup default stable-x86_64-pc-windows-msvc`

```bash
# Build from source
git clone https://github.com/lode-rs/lode
cd lode
cargo build -p lode-cli
cargo build -p lode-lsp

# Run setup
./target/debug/lode setup
```

### Quick Start

```bash
# Initialize a new project
lode init my-project

# Check naming conventions
lode check

# Scan for secrets
lode scan secrets

# Run project health audit
lode health

# Start the TUI dashboard
lode serve

# Start the MCP server (for AI agents)
lode mcp

# Start the LSP server (for editors)
lode lsp
```

### Basic Workflow

```
1. lode init my-project    # Create project scaffold
2. cd my-project
3. lode check              # Verify conventions
4. lode scan secrets       # Security scan
5. lode health             # Project health audit
6. lode serve              # Monitor via TUI (optional)
```

## Development Workflow

### Standard Commands

```bash
lode build    # Build project
lode test     # Run tests
lode fmt      # Format code
lode lint     # Run linter
lode check    # Check conventions
lode verify   # Full verification
lode clean    # Clean artifacts
lode fresh    # Clean + rebuild
lode ship     # Verify + prepare release
```

### Git Integration

```bash
lode git branch feat my-feature    # Create conventional branch
lode git commit "Add feature"      # Create conventional commit
lode git tag v0.2.0                # Create tag
lode git changelog                 # Generate changelog
lode hooks install                 # Install git hooks
```

### Environment Management

```bash
lode env check    # Check for drift/missing values
lode env add KEY value   # Add environment variable
lode env sync     # Synchronize .env file
```

### Package Management

```bash
lode pkg list         # Detect package manager
lode pkg outdated     # List outdated deps
lode pkg audit        # Vulnerability audit
lode pkg update dep   # Update dependency
```

### Release

```bash
lode release --bump patch    # Bump version (dry run)
lode release --bump minor    # Minor version bump
lode release --bump major    # Major version bump
lode release --rollback      # Rollback version
```

### Time Tracking

```bash
lode time today     # Today's summary
lode time report    # Session report
lode time clear     # Clear history
```

## Extension Guides

### VS Code (vscode-lode)

1. Open the `extensions/vscode-lode` directory
2. Run `npm install`
3. Press F5 to launch debug session
4. Commands: `Lode: Check`, `Lode: Scan`, `Lode: Init`, `Lode: Sync`, `Lode: Status`
5. Settings: `lode.binaryPath`, `lode.enableDiagnostics`, `lode.enableDecorations`

### Neovim (lode.nvim)

```lua
-- Using lazy.nvim
{
  'lode-rs/lode',
  dir = '/path/to/lode/extensions/lode.nvim',
  opts = {
    bin_path = 'lode',
    enable_diagnostics = true,
    check_on_save = true,
    keymaps = {
      check = '<leader>lc',
      scan  = '<leader>ls',
    }
  }
}
```

### Zed (zed-lode)

1. Copy `extensions/zed-lode` to `~/.config/zed/extensions/`
2. Run `cargo build --release -p zed-lode` in the extension dir
3. Slash commands: `/check`, `/scan`, `/status`, `/init`

## Fuzz Testing

```bash
cargo install cargo-fuzz
cargo fuzz run validated_root -- -max_total_time=30
cargo fuzz run process_validation -- -max_total_time=30
```

## Coverage

```bash
cargo install cargo-llvm-cov
cargo llvm-cov --workspace --lcov --output-path lcov.info
```
