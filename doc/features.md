# Features

## Overview

This document tracks all features of the knowledge-index CLI application.

## Feature List

| Feature | Description | Added |
|---------|-------------|-------|
| **Directory Indexing** | Index any directory (code repos, Obsidian vaults, notes) | 2026-02-06 |
| **Full-Text Search** | FTS5-powered search across all indexed content | 2026-02-06 |
| **Incremental Updates** | Smart re-indexing that only processes changed files | 2026-02-06 |
| **TUI App Mode** | Interactive terminal UI with search and repo management | 2026-02-06 |
| **CLI Mode** | Command-line interface with subcommands | 2026-02-06 |
| **Repository Management** | List, update, and remove indexed repositories | 2026-02-06 |
| **Configuration System** | TOML-based config with sensible defaults | 2026-02-06 |
| **JSON Output** | Machine-readable output for scripting/automation | 2026-02-06 |
| **Search Filters** | Filter by repository name and file type | 2026-02-06 |
| **Progress Indicators** | Visual progress bars during indexing | 2026-02-06 |
| **Binary Detection** | Automatic skipping of binary files | 2026-02-06 |
| **Gitignore Support** | Respects .gitignore patterns during indexing | 2026-02-06 |
| **File Watcher (Core)** | Filesystem monitoring infrastructure for auto-updates | 2026-02-07 |
| **MCP Server** | Model Context Protocol server for AI tool integration | 2026-02-07 |
| **MCP Search Tool** | Search indexed content via MCP | 2026-02-07 |
| **MCP List Repos Tool** | List repositories via MCP | 2026-02-07 |
| **MCP Get File Tool** | Retrieve file content via MCP | 2026-02-07 |
| **MCP Get Context Tool** | Get line context around specific lines via MCP | 2026-02-07 |

## Planned Features

- Background file watching (TUI integration)
- Vector search / semantic similarity
- Markdown frontmatter parsing
- Remote sync capabilities
