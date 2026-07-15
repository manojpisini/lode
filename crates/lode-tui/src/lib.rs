//! Terminal UI (TUI) dashboard for LODE.
//!
//! Renders a multi-pane dashboard with overview, metrics, time tracking,
//! activity, dependencies, files, and registry views using ratatui.
#![deny(unsafe_code)]

pub mod app;
pub mod panes;
pub mod theme;
pub mod widgets;
