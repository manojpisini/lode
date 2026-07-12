# lode-tui

[![crates.io](https://img.shields.io/crates/v/lode-tui.svg)](https://crates.io/crates/lode-tui)
[![docs.rs](https://img.shields.io/docsrs/lode-tui)](https://docs.rs/lode-tui/latest/lode_tui/)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Terminal UI dashboard for [LODE](https://github.com/manojpisini/lode).  
Seven panes with custom ratatui widgets providing a real-time project dashboard.

## Installation

```toml
[dependencies]
lode-tui = "0.1"
```

Or build from source:

```bash
git clone https://github.com/manojpisini/lode.git
cd lode
cargo build -p lode-tui
```

## Features

### Panes

| Pane | Description |
|---|---|
| **Status** | Overview of project state, daemon status, and quick actions |
| **Files** | Real-time file watcher events and file system changes |
| **Metrics** | Project metrics, trends, and health scores |
| **Activity** | Recent activity log with timestamps |
| **Commands** | Quick command palette for common operations |
| **Plugins** | Plugin status and management |
| **Config** | Configuration viewer and editor |

### Widgets

- **BarChart** — horizontal bar charts for metric visualization
- **Heatmap** — activity heatmaps for time-based data
- **ScoreRing** — circular score indicators for project health
- **Sparkline** — inline sparkline charts for trends
- **StatusBar** — multi-section status bar with contextual information

### Theming

- Dark theme with configurable accent colors
- Rounded or sharp border styles
- Color-aware pane rendering

### IPC

- Connects to lode-daemon via IPC for live event streaming
- Configurable refresh interval (default: 1s)

## Related crates

- [lode-core](https://crates.io/crates/lode-core) — Core library
- [lode-daemon](https://crates.io/crates/lode-daemon) — File watcher daemon
- [lode-cli](https://crates.io/crates/lode-cli) — CLI binary with `lode serve` command

## License

MIT
