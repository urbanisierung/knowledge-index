# knowledge-index

A fast, local-first CLI for indexing code repositories and knowledge bases, enabling AI-powered search across all your projects.

## Motivation

Modern developers and knowledge workers maintain dozens of repositories, documentation sites, and note collections (like Obsidian vaults). When working with AI assistants, finding the right context across these scattered sources is challenging.

**knowledge-index** solves this by:
- **Indexing everything locally** — Code, markdown, configs across all your projects
- **Enabling instant full-text search** — SQLite FTS5 provides sub-millisecond queries
- **Providing AI-ready output** — JSON mode and MCP server for AI assistant integration
- **Working offline** — No cloud dependencies, your data stays local

## Prerequisites

- Rust 1.88+ (install via [rustup](https://rustup.rs/))
- Docker (optional, for running CI checks locally)

## Quickstart

```bash
# Clone and build
git clone https://github.com/urbanisierung/knowledge-index.git
cd knowledge-index
cargo build --release

# Index a project
./target/release/knowledge-index index /path/to/your/project

# Search across all indexed content
./target/release/knowledge-index search "async function"

# Launch interactive TUI
./target/release/knowledge-index

# Or install globally
cargo install --path .
```

## Usage

The CLI supports two modes:

### App Mode (TUI)

```bash
knowledge-index
```

Launches a full-screen interactive interface for searching and managing indexed repositories.

### CLI Mode

```bash
# Index current directory
knowledge-index index .

# Search for code
knowledge-index search "database connection"

# List all indexed repos
knowledge-index list

# Get JSON output for scripting
knowledge-index search "TODO" --json
```

Run `knowledge-index --help` for all available commands.

## AI Integration (MCP)

Start the MCP server to use knowledge-index with AI assistants:

```bash
knowledge-index mcp
```

### GitHub Copilot CLI

Add to your MCP servers configuration:

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

### Claude Desktop

Add to `~/.config/claude/claude_desktop_config.json` (Linux/macOS) or `%APPDATA%\Claude\claude_desktop_config.json` (Windows):

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

See [MCP Integration Guide](doc/mcp-integration.md) for detailed setup and available tools.

## Development

This project uses a Makefile for development workflows. Docker is used to run CI checks locally, ensuring consistency with GitHub Actions.

```bash
# Run full CI pipeline (recommended before pushing)
make ci

# Run quick checks (format + clippy only)
make ci-quick

# Check minimum supported Rust version
make ci-msrv

# Individual CI steps
make ci-format      # Check formatting
make ci-clippy      # Run clippy lints
make ci-test        # Run tests
make ci-doc         # Build documentation

# Local development (uses local Rust toolchain)
make build          # Build debug
make release        # Build release
make test           # Run tests
make fmt            # Format code
make lint           # Run clippy
make clean          # Clean artifacts
```

Run `make help` to see all available commands.

## Documentation

- [Features](doc/features.md) — Feature overview
- [Documentation](doc/documentation.md) — Detailed usage guide
- [MCP Integration](doc/mcp-integration.md) — AI assistant setup guide
- [Roadmap](doc/roadmap.md) — Implementation roadmap
- [Progress](doc/progress.md) — Changelog

## License

MIT
