# lode-daemon

[![crates.io](https://img.shields.io/crates/v/lode-daemon.svg)](https://crates.io/crates/lode-daemon)
[![docs.rs](https://img.shields.io/docsrs/lode-daemon)](https://docs.rs/lode-daemon/latest/lode_daemon/)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Background file watcher daemon for [LODE](https://github.com/manojpisini/lode).

## Installation

```toml
[dependencies]
lode-daemon = "0.1"
```

Or build from source:

```bash
git clone https://github.com/manojpisini/lode.git
cd lode
cargo build -p lode-daemon
```

## Features

- **Filesystem watching** — monitors files and directories for create, modify, and delete events
- **IPC communication** — JSON-based IPC protocol with token-based authentication
- **Idle watchdog** — automatic shutdown after configurable idle timeout (default: 300s)
- **Debounced events** — configurable debounce window (default: 150ms) to batch rapid changes
- **Watcher management** — add/remove watchers at runtime via IPC commands
- **Event logging** — ring buffer of recent events with structured metadata
- **State persistence** — daemon state (watchers, counters) saved to disk for recovery
- **TCP fallback** — automatic TCP port fallback when Unix sockets are unavailable
- **Pause/resume** — temporarily suspend watching without stopping the process
- **Foreground mode** — run in foreground for debugging

## Related crates

- [lode-core](https://crates.io/crates/lode-core) — Core library
- [lode-cli](https://crates.io/crates/lode-cli) — CLI binary with daemon management commands

## License

MIT
