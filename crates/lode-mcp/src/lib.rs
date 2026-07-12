//! MCP (Model Context Protocol) server for LODE.
//!
//! Exposes 38+ tools, 8 resources, and 3 prompts over the MCP protocol,
//! enabling AI agents and editors to interact with LODE project management,
//! conventions, secrets, git, package management, and more.
#![deny(unsafe_code)]

pub mod error;
pub mod prompts;
pub mod resources;
pub mod schema;
pub mod server;
pub mod tools;
pub mod transport;
