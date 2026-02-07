//! MCP (Model Context Protocol) server implementation.
//!
//! This module provides an MCP server that allows AI assistants to search
//! and access indexed content.

mod server;

pub use server::run_mcp_server;
