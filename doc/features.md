# Features

## Overview

kdex indexes your code, docs, notes, and wikis locally, making everything searchable for you and your AI assistants.

## Highlights

| What | Why It Matters |
|------|----------------|
| **🔍 Instant Search** | SQLite FTS5 gives sub-millisecond queries across all your knowledge |
| **🤖 AI-Ready** | MCP server lets GitHub Copilot, Claude, and Ollama search your files |
| **📁 Universal** | Works with code repos, Obsidian vaults, wikis, any markdown |
| **🔒 Local-First** | Your data never leaves your machine. Works offline. |
| **⚡ Fast** | Index 100k files in seconds, search in milliseconds |
| **🖥️ Interactive TUI** | Full-screen interface with preview panel |

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
| **Gemini CLI Integration** | MCP integration docs for Gemini CLI | 2026-02-11 |
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
| **Remote GitHub Repos** | Clone and index remote GitHub repositories | 2026-02-09 |
| **Background Sync** | Auto-sync remote repos to stay up-to-date | 2026-02-09 |
| **SSH/Token Auth** | Support SSH agent and GitHub tokens for private repos | 2026-02-09 |
| **Config Export** | Export configuration to YAML for backup/migration | 2026-02-09 |
| **Config Import** | Import configuration with merge support | 2026-02-09 |
| **Shallow Clone** | Optional shallow clone for faster setup | 2026-02-09 |
| **Remote Cleanup** | Auto-delete cloned repos when removed from index | 2026-02-09 |
| **Default Search** | Search without typing `search` - just `kdex "query"` | 2026-02-11 |
| **Short Flags** | `-r`, `-t`, `-l`, `-s`, `-H`, `-g` for faster searching | 2026-02-11 |
| **Shell Completions** | Generate completions for bash, zsh, fish, PowerShell | 2026-02-11 |
| **Fuzzy Search** | Typo-tolerant search using Jaro-Winkler similarity | 2026-02-11 |
| **Regex Search** | Pattern matching with regular expressions | 2026-02-11 |
| **Backlinks Discovery** | Find files linking to a target (`[[wiki-links]]`) | 2026-02-11 |
| **Tags Browser** | List all tags from indexed markdown frontmatter | 2026-02-11 |
| **Context Builder** | Build AI prompts from search results with token limits | 2026-02-11 |
| **Knowledge Statistics** | View index stats (files, repos, tags, links, storage) | 2026-02-11 |
| **Tags/Links Indexing** | Store tags and wiki-links during indexing for backlinks | 2026-02-11 |
| **Graph Visualization** | Export knowledge graph in DOT (Graphviz) or JSON format | 2026-02-11 |
| **Health Diagnostics** | Find orphan files, broken links, and health score | 2026-02-11 |
| **Vault Auto-Detection** | Auto-detect Obsidian, Logseq, Dendron vaults | 2026-02-11 |
| **Search History** | Navigate previous searches with Up/Down arrows in TUI | 2026-02-11 |

## Planned Features

- Background file watching (TUI integration)
