# Documentation

## Installation

```bash
cargo install --path .
```

Or build from source:
```bash
cargo build --release
# Binary will be at ./target/release/knowledge-index
```

## Quick Start

1. **Index a directory:**
   ```bash
   knowledge-index index /path/to/project
   ```

2. **Search indexed content:**
   ```bash
   knowledge-index search "your query"
   ```

3. **Launch interactive TUI:**
   ```bash
   knowledge-index
   ```

## Usage

### App Mode (TUI)

```bash
knowledge-index
```

Launches the full-screen interactive interface.

**Keyboard Shortcuts:**

| Key | Action |
|-----|--------|
| `Tab` | Switch between Search and Repos views |
| `?` | Toggle help overlay |
| `q` | Quit (when search is empty) |
| `Ctrl+C` | Force quit |
| `↑`/`k` | Move up |
| `↓`/`j` | Move down |
| `Enter` | Select / Open file |
| `Esc` | Clear search / Go back |
| `Ctrl+U` | Clear search input |
| `d` | Delete repository (in Repos view) |
| `r` | Refresh list (in Repos view) |

### CLI Mode

```bash
knowledge-index [COMMAND] [OPTIONS]
```

Global options:
- `--json` - Output as JSON for scripting
- `--quiet` - Suppress non-error output
- `--no-color` - Disable colored output
- `-v, --verbose` - Enable verbose output

## Commands

### `index`

Index a directory (code repository or knowledge base).

```bash
knowledge-index index [PATH] [OPTIONS]

# Examples
knowledge-index index                    # Index current directory
knowledge-index index ~/projects/myapp   # Index specific project
knowledge-index index ~/notes --name obsidian-vault
```

Options:
- `--name <NAME>` - Custom name for the repository

### `search`

Search indexed content.

```bash
knowledge-index search <QUERY> [OPTIONS]

# Examples
knowledge-index search "async fn"
knowledge-index search "database connection" --repo api-service
knowledge-index search "TODO" --type markdown
knowledge-index search "config" --limit 50
knowledge-index search "authentication logic" --semantic
knowledge-index search "error handling" --hybrid
```

Options:
- `--repo <NAME>` - Filter by repository name
- `--type <TYPE>` - Filter by file type (rust, python, markdown, etc.)
- `--limit <N>` - Maximum results (default: 20)
- `--semantic` - Use vector/embedding search (requires `enable_semantic_search = true`)
- `--hybrid` - Combine lexical + semantic search with RRF fusion
- `--lexical` - Use full-text search only (default)

### `list`

List all indexed repositories.

```bash
knowledge-index list

# Output:
# ● my-project    │    142 files │   5.2 MB │ 2 hours ago
# ● obsidian-vault│    891 files │  12.3 MB │ just now
```

### `update`

Update an existing index.

```bash
knowledge-index update [PATH]
knowledge-index update --all    # Update all repositories
```

### `remove`

Remove a repository from the index.

```bash
knowledge-index remove /path/to/project
knowledge-index remove . --force  # Skip confirmation
```

### `config`

Show or edit configuration.

```bash
knowledge-index config                         # Show current config
knowledge-index config max_file_size_mb 20     # Set a value
knowledge-index config --reset                 # Reset to defaults
```

## Configuration

Configuration is stored at:
- **Linux:** `~/.config/knowledge-index/config.toml`
- **macOS:** `~/Library/Application Support/knowledge-index/config.toml`
- **Windows:** `%APPDATA%\knowledge-index\config.toml`

### Config Options

| Option | Default | Description |
|--------|---------|-------------|
| `max_file_size_mb` | 10 | Skip files larger than this |
| `color_enabled` | true | Enable colored output |
| `batch_size` | 100 | Files per database transaction |
| `watcher_debounce_ms` | 500 | File watcher debounce delay |
| `ignore_patterns` | [".git", "node_modules", ...] | Patterns to ignore |
| `enable_semantic_search` | false | Enable vector embeddings for semantic search |
| `embedding_model` | "all-MiniLM-L6-v2" | Embedding model to use |
| `default_search_mode` | "lexical" | Default search mode (lexical, semantic, hybrid) |

## Database

The index database is stored at:
- **Linux:** `~/.config/knowledge-index/index.db`
- **macOS:** `~/Library/Application Support/knowledge-index/index.db`
- **Windows:** `%APPDATA%\knowledge-index\index.db`

It uses SQLite with FTS5 for full-text search. When semantic search is enabled, embeddings are stored in a separate table.

## Search Modes

### Lexical (Default)
Full-text search using SQLite FTS5. Best for exact keyword matches, code symbols, and specific terms.

- Simple words: `function database`
- Phrases: `"exact phrase"`
- Prefix matching: `func*`
- Boolean: `config AND database`
- Exclusion: `config NOT test`

### Semantic (--semantic)
Vector-based search using embedding similarity. Best for conceptual queries where exact keywords may not match.

```bash
knowledge-index search "how to handle authentication" --semantic
```

Requires `enable_semantic_search = true` in config. On first use, downloads the embedding model (~22MB).

### Hybrid (--hybrid)
Combines lexical and semantic search using Reciprocal Rank Fusion (RRF). Provides the best of both approaches.

```bash
knowledge-index search "error handling patterns" --hybrid
```
