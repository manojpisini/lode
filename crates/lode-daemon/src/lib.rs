//! Background file watcher daemon for LODE.
//!
//! Monitors project directories for file system events (create, modify, rename,
//! delete) and applies configured actions such as signature stamps and secret
//! scanning. Communicates with the CLI and TUI via Unix domain sockets (Linux/macOS)
//! or TCP (Windows fallback).
#![deny(unsafe_code)]

pub mod handlers;
pub mod idle;
pub mod ipc;
pub mod state;
pub mod watcher;

pub use handlers::*;
pub use idle::*;
pub use ipc::*;
pub use state::*;
pub use watcher::*;
