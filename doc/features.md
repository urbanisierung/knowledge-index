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
| **Semantic Search** | Vector-based search using MiniLM embeddings | 2026-02-08 |
| **Hybrid Search** | Combined lexical + semantic search with RRF fusion | 2026-02-08 |
| **Text Chunking** | Smart chunking for embedding large files | 2026-02-08 |
| **Search Mode Selection** | CLI flags for --semantic, --hybrid, --lexical | 2026-02-08 |
| **Group by Repository** | `--group-by-repo` flag clusters search results | 2026-02-08 |
| **Markdown Frontmatter** | Parse YAML frontmatter (title, tags) | 2026-02-08 |
| **Markdown Headings** | Extract and index heading hierarchy | 2026-02-08 |
| **Wiki-links** | Extract `[[link]]` style wiki links | 2026-02-08 |
| **Rebuild Embeddings** | `rebuild-embeddings` command for regeneration | 2026-02-08 |
| **Watch Command** | `watch` command for live file monitoring | 2026-02-08 |
| **Embedding Progress** | Progress indicator during embedding generation | 2026-02-08 |
| **TUI Preview Pane** | Show file content preview for selected search result | 2026-02-08 |
| **TUI Loading States** | Animated spinner overlay during operations | 2026-02-08 |
| **Delete Confirmation** | Confirmation dialog before repo deletion | 2026-02-08 |
| **Platform Limits Check** | Warn about Linux inotify limits before watching | 2026-02-08 |
| **Welcome Screen** | First-run welcome screen with getting started guide | 2026-02-08 |
| **Debug Mode** | `--debug` flag for verbose output with backtraces | 2026-02-08 |
| **Shell Aliases** | Suggested shell aliases in --help output | 2026-02-08 |
| **Markdown Syntax Stripping** | Config option to strip markdown for cleaner FTS | 2026-02-08 |
| **Code Block Indexing** | Config option to index code blocks with language tags | 2026-02-08 |
| **CI Pipeline** | GitHub Actions for testing on Linux/macOS/Windows | 2026-02-08 |
| **Release Automation** | Cross-platform binary builds and GitHub releases | 2026-02-08 |
| **Unit Tests** | 18 unit tests covering config, search, and markdown | 2026-02-08 |
| **Integration Tests** | CLI command tests for help, version, config, search | 2026-02-08 |

## Planned Features

- Background file watching (TUI integration)
- Remote sync capabilities
