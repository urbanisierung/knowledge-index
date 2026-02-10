# Documentation

## Installation

```bash
cargo install --path .
```

Or build from source:
```bash
cargo build --release
# Binary will be at ./target/release/kdex
```

## Quick Start

1. **Index a directory:**
   ```bash
   kdex index /path/to/project
   ```

2. **Search indexed content:**
   ```bash
   kdex search "your query"
   ```

3. **Launch interactive TUI:**
   ```bash
   kdex
   ```

## Usage

### App Mode (TUI)

```bash
kdex
```

Launches the full-screen interactive interface.

**Keyboard Shortcuts:**

| Key | Action |
|-----|--------|
| `Tab` | Switch between Search and Repos views |
| `?` | Toggle help overlay |
| `Ctrl+Q` | Quit application |
| `Ctrl+C` | Force quit |
| `↑`/`Ctrl+K` | Move up |
| `↓`/`Ctrl+J` | Move down |
| `Enter` | Select / Open file |
| `Esc` | Clear search / Go back |
| `Ctrl+P` | Toggle preview panel |
| `Ctrl+O` | Open file in editor |
| `Ctrl+U` | Clear search input |
| `d` | Delete repository (in Repos view) |
| `r` | Refresh list (in Repos view) |

### CLI Mode

```bash
kdex [COMMAND] [OPTIONS]
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
kdex index [PATH] [OPTIONS]

# Examples
kdex index                    # Index current directory
kdex index ~/projects/myapp   # Index specific project
kdex index ~/notes --name obsidian-vault
```

Options:
- `--name <NAME>` - Custom name for the repository

### `search`

Search indexed content.

```bash
kdex search <QUERY> [OPTIONS]

# Examples
kdex search "async fn"
kdex search "database connection" --repo api-service
kdex search "TODO" --type markdown
kdex search "config" --limit 50
kdex search "authentication logic" --semantic
kdex search "error handling" --hybrid
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
kdex list

# Output:
# ● my-project    │    142 files │   5.2 MB │ 2 hours ago
# ● obsidian-vault│    891 files │  12.3 MB │ just now
```

### `update`

Update an existing index.

```bash
kdex update [PATH]
kdex update --all    # Update all repositories
```

### `remove`

Remove a repository from the index.

```bash
kdex remove /path/to/project
kdex remove . --force  # Skip confirmation
```

### `config`

Show or edit configuration.

```bash
kdex config                         # Show current config
kdex config max_file_size_mb 20     # Set a value
kdex config --reset                 # Reset to defaults
```

### `mcp`

Start the MCP (Model Context Protocol) server for AI assistant integration.

```bash
kdex mcp
```

The MCP server allows AI tools like GitHub Copilot CLI, Claude Desktop, or other MCP-compatible clients to search and retrieve content from your indexed repositories. See [MCP Integration Guide](mcp-integration.md) for detailed setup instructions.

## Configuration

Configuration is stored at:
- **Linux:** `~/.config/kdex/config.toml`
- **macOS:** `~/Library/Application Support/kdex/config.toml`
- **Windows:** `%APPDATA%\kdex\config.toml`

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
- **Linux:** `~/.config/kdex/index.db`
- **macOS:** `~/Library/Application Support/kdex/index.db`
- **Windows:** `%APPDATA%\kdex\index.db`

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
kdex search "how to handle authentication" --semantic
```

Requires `enable_semantic_search = true` in config. On first use, downloads the embedding model (~22MB).

### Hybrid (--hybrid)
Combines lexical and semantic search using Reciprocal Rank Fusion (RRF). Provides the best of both approaches.

```bash
kdex search "error handling patterns" --hybrid
```

## Remote Repository Support

kdex can clone and sync remote GitHub repositories, keeping them up-to-date automatically.

### Adding Remote Repositories

```bash
# Add from GitHub (shorthand)
kdex add --remote owner/repo

# Add with full URL
kdex add --remote https://github.com/owner/repo.git

# Specify branch
kdex add --remote owner/repo --branch develop

# Use a custom name
kdex add --remote owner/repo --name my-docs

# Shallow clone (faster, less disk space)
kdex add --remote owner/repo --shallow
```

Remote repositories are cloned to:
- **Linux:** `~/.config/kdex/repos/<owner>/<repo>/`
- **macOS:** `~/Library/Application Support/kdex/repos/<owner>/<repo>/`
- **Windows:** `%APPDATA%\kdex\repos\<owner>\<repo>\`

### Authentication

For private repositories, kdex supports:

1. **SSH Agent** (recommended): If you have an SSH agent running with your GitHub key
2. **Environment Variable**: Set `KDEX_GITHUB_TOKEN` or `GITHUB_TOKEN` with a personal access token

### Syncing Remote Repositories

```bash
# Sync all remote repositories
kdex sync

# Sync a specific repository
kdex sync owner/repo
```

Background sync also runs automatically during search operations to keep content fresh.

### Removing Remote Repositories

When you remove a remote repository, the cloned directory is also deleted:

```bash
kdex remove owner/repo
```

## Configuration Import/Export

Easily migrate your kdex setup between machines.

### Export Configuration

```bash
# Export to stdout (YAML)
kdex config export

# Export to a file
kdex config export -o kdex-backup.yaml

# Export only remote repositories (portable)
kdex config export --remotes-only

# Include local repos (paths may not work on other machines)
kdex config export --include-local
```

### Import Configuration

```bash
# Import from file
kdex config import kdex-backup.yaml

# Merge with existing config (don't overwrite)
kdex config import kdex-backup.yaml --merge

# Import from stdin
cat kdex-backup.yaml | kdex config import -

# Skip confirmation prompts
kdex config import kdex-backup.yaml --yes
```

### Portable Config Format

```yaml
version: 1
repositories:
  - type: remote
    url: https://github.com/owner/repo.git
    branch: main
settings:
  max_file_size_mb: 10
  enable_semantic_search: true
  default_search_mode: hybrid
```
