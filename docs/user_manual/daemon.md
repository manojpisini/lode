# Daemon Usage Guide

## Overview

The LODE daemon runs in the background, watching files and handling IPC commands.

## Starting the Daemon

```bash
lode daemon start
```

## Checking Status

```bash
lode daemon status
```

Output:
```
Daemon is running
PID: 12345
Uptime: 2h 15m
Events tracked: 47
```

## Viewing the Log

```bash
lode daemon log
```

Follow the log in real-time:

```bash
lode daemon log --tail
```

## Stopping the Daemon

```bash
lode daemon stop
```

## Daemon Features

- **File watching:** Monitors project files for changes
- **IPC:** Handles commands from CLI over authenticated IPC
- **Idle watchdog:** Auto-shutdown after period of inactivity
- **State persistence:** Saves and restores state on restart
- **JSON output:** Supports `--json` flag for machine-readable status
