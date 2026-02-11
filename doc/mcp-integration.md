# MCP Integration Guide

This document explains how to integrate `kdex` with AI assistants using the Model Context Protocol (MCP).

## Overview

`kdex` provides an MCP server that allows AI assistants to search and retrieve content from your indexed repositories and knowledge bases. The server communicates over stdio, making it compatible with local AI tools.

## Quick Start

Start the MCP server:

```bash
kdex mcp
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
    "kdex": {
      "command": "kdex",
      "args": ["mcp"]
    }
  }
}
```

### GitHub Copilot CLI

GitHub Copilot CLI supports MCP servers for enhanced context. Add kdex to your MCP configuration:

**Linux/macOS:** `~/.config/github-copilot/mcp.json`
**Windows:** `%APPDATA%\GitHub Copilot\mcp.json`

```json
{
  "mcpServers": {
    "kdex": {
      "command": "kdex",
      "args": ["mcp"]
    }
  }
}
```

After configuration, Copilot CLI can search your indexed repositories, retrieve file contents, and get code context automatically.

### Gemini CLI

[Gemini CLI](https://geminicli.com/) supports MCP servers for extending its capabilities. Add kdex to your settings:

**Location:** `~/.gemini/settings.json`

```json
{
  "mcpServers": {
    "kdex": {
      "command": "kdex",
      "args": ["mcp"],
      "timeout": 30000
    }
  }
}
```

**Configuration options:**
- `command`: Path to kdex binary (ensure it's in your PATH)
- `args`: Arguments to pass (just `["mcp"]` for the MCP server)
- `timeout`: Request timeout in milliseconds (default: 600000)
- `trust`: Set to `true` to skip confirmation prompts for kdex tools

**Usage in Gemini CLI:**

Once configured, you can ask Gemini to search your indexed content:

```
> Search my codebase for authentication patterns
> Find all markdown files about API design
> Get the contents of src/main.rs from my-project
```

Gemini will automatically use kdex's MCP tools (`search`, `list_repos`, `get_file`, `get_context`) to answer your questions with context from your indexed repositories.

### Shell Aliases

Add these to your `.bashrc` or `.zshrc` for quick access:

```bash
# Quick search with JSON output
alias ki-search='kdex search --json'

# Start MCP server
alias ki-mcp='kdex mcp'

# List indexed repos
alias ki-list='kdex list'
```

## Prerequisites

Before using the MCP server, ensure you have indexed at least one repository:

```bash
# Index your project
cd ~/projects/my-app
kdex index

# Or index any directory
kdex index ~/Documents/notes
```

## Troubleshooting

### No Results Returned

1. Check if repositories are indexed: `kdex list`
2. Verify the search query syntax
3. Try a broader search term

### Server Not Starting

1. Ensure the binary is in your PATH
2. Check stderr output for error messages
3. Verify the database exists at the expected location:
   - Linux: `~/.config/kdex/index.db`
   - macOS: `~/Library/Application Support/kdex/index.db`
   - Windows: `%APPDATA%\kdex\index.db`

### Connection Issues

1. Ensure only one MCP client connects at a time
2. Check that stdin/stdout are not being redirected elsewhere
3. Verify the AI tool's MCP configuration

## Best Practices for AI Assistants

When using kdex with AI assistants:

1. **Start with `list_repos`** to understand what content is available
2. **Use `search` for discovery** with broad queries, then narrow down
3. **Use `get_file` for full context** when you find relevant results
4. **Use `get_context` for targeted context** around specific lines
5. **Respect `truncated` flags** and use pagination or `get_file` for more content
