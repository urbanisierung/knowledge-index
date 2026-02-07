# MCP Integration Guide

This document explains how to integrate `knowledge-index` with AI assistants using the Model Context Protocol (MCP).

## Overview

`knowledge-index` provides an MCP server that allows AI assistants to search and retrieve content from your indexed repositories and knowledge bases. The server communicates over stdio, making it compatible with local AI tools.

## Quick Start

Start the MCP server:

```bash
knowledge-index mcp
```

The server will output to stderr for logging and use stdout exclusively for MCP protocol messages.

## Available Tools

### `search`

Search indexed content across all repositories.

**Parameters:**
- `query` (required): Search query (keywords, phrases, or patterns)
- `limit` (optional): Maximum results to return (default: 10, max: 50)
- `repo` (optional): Filter by repository name
- `file_type` (optional): Filter by file type (e.g., 'rust', 'markdown', 'python')

**Example Response:**
```json
{
  "results": [
    {
      "file": "/home/user/projects/my-app/src/auth.rs",
      "repo": "my-app",
      "snippet": "...fn authenticate_user(credentials: &Credentials)...",
      "score": 0.95
    }
  ],
  "total": 15,
  "query": "authenticate user",
  "truncated": false,
  "hint": null
}
```

### `list_repos`

List all indexed repositories with their status and file counts.

**Parameters:** None

**Example Response:**
```json
{
  "repositories": [
    {
      "name": "my-app",
      "path": "/home/user/projects/my-app",
      "file_count": 234,
      "status": "ready",
      "last_indexed": "2024-01-15T10:30:00Z"
    }
  ],
  "total": 3
}
```

### `get_file`

Get the full content of a specific file.

**Parameters:**
- `path` (required): Absolute path to the file
- `max_chars` (optional): Maximum characters to return (default: 50000)

**Example Response:**
```
File: /home/user/projects/my-app/src/main.rs
Type: rs
Truncated: false

fn main() {
    println!("Hello, world!");
}
```

### `get_context`

Get lines of context around a specific line number in a file.

**Parameters:**
- `path` (required): Absolute path to the file
- `line` (required): Line number to center the context around
- `context_lines` (optional): Number of context lines before and after (default: 10)

**Example Response:**
```
File: /home/user/projects/my-app/src/auth.rs
Lines 35-55 (centered on line 45):

  35 | impl AuthService {
  36 |     pub fn new(config: &Config) -> Self {
  37 |         Self {
...
  45 |     pub fn authenticate(&self, user: &str, pass: &str) -> Result<Token> {
...
```

## Integration Examples

### Claude Desktop

Add to your `claude_desktop_config.json` (typically at `~/.config/claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "knowledge-index": {
      "command": "knowledge-index",
      "args": ["mcp"]
    }
  }
}
```

### GitHub Copilot CLI

The MCP server can be used with any tool that supports the MCP protocol over stdio.

### Shell Aliases

Add these to your `.bashrc` or `.zshrc` for quick access:

```bash
# Quick search with JSON output
alias ki-search='knowledge-index search --json'

# Start MCP server
alias ki-mcp='knowledge-index mcp'

# List indexed repos
alias ki-list='knowledge-index list'
```

## Prerequisites

Before using the MCP server, ensure you have indexed at least one repository:

```bash
# Index your project
cd ~/projects/my-app
knowledge-index index

# Or index any directory
knowledge-index index ~/Documents/notes
```

## Troubleshooting

### No Results Returned

1. Check if repositories are indexed: `knowledge-index list`
2. Verify the search query syntax
3. Try a broader search term

### Server Not Starting

1. Ensure the binary is in your PATH
2. Check stderr output for error messages
3. Verify the database exists at the expected location:
   - Linux: `~/.config/knowledge-index/index.db`
   - macOS: `~/Library/Application Support/knowledge-index/index.db`
   - Windows: `%APPDATA%\knowledge-index\index.db`

### Connection Issues

1. Ensure only one MCP client connects at a time
2. Check that stdin/stdout are not being redirected elsewhere
3. Verify the AI tool's MCP configuration

## Best Practices for AI Assistants

When using knowledge-index with AI assistants:

1. **Start with `list_repos`** to understand what content is available
2. **Use `search` for discovery** with broad queries, then narrow down
3. **Use `get_file` for full context** when you find relevant results
4. **Use `get_context` for targeted context** around specific lines
5. **Respect `truncated` flags** and use pagination or `get_file` for more content
