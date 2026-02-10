<p align="center">
  <img src="https://img.shields.io/badge/rust-1.88+-orange?logo=rust" alt="Rust 1.88+">
  <img src="https://img.shields.io/badge/license-MIT-blue" alt="MIT License">
  <img src="https://img.shields.io/badge/platform-linux%20%7C%20macos%20%7C%20windows-lightgrey" alt="Platform">
</p>

<h1 align="center">kdex</h1>

<p align="center">
  <strong>All your knowledge. One search. AI-ready.</strong>
</p>

<p align="center">
  Index your code, docs, notes, and wikis locally.<br>
  Let AI assistants search everything you know.
</p>

---

## The Problem

You have knowledge everywhere:
- **Code** across dozens of repositories
- **Notes** in Obsidian, Logseq, or markdown files
- **Docs** scattered in wikis and READMEs

When you ask an AI assistant for help, it has no idea what's in YOUR files. You end up copy-pasting context manually.

## The Solution

```bash
# Index everything
kdex index ~/code/my-project
kdex index ~/Documents/obsidian-vault
kdex index ~/wiki

# Search instantly
kdex search "how to deploy"

# AI assistants can now search your knowledge
kdex mcp  # Start MCP server for Copilot, Claude, Ollama
```

**kdex** creates a local search index that AI tools can query. Your data never leaves your machine.

---

## ✨ Features

| Feature | Description |
|---------|-------------|
| 🔍 **Instant Search** | SQLite FTS5 gives sub-millisecond full-text search |
| 🤖 **AI-Ready** | MCP server for GitHub Copilot, Claude, Ollama |
| 📁 **Universal** | Code repos, Obsidian vaults, wikis, any markdown |
| 🔒 **Local-First** | Your data stays on your machine. Always. |
| ⚡ **Fast** | Index 100k files in seconds, search in milliseconds |
| 🌐 **Remote Repos** | Add GitHub repos by URL—auto-cloned, auto-synced |
| 📦 **Portable** | Export/import config for easy machine migration |
| 🖥️ **Interactive TUI** | Full-screen interface with preview panel |

---

## 🚀 Quickstart

```bash
# Install
cargo install kdex

# Index your project
kdex index .

# Add a GitHub repo
kdex add --remote owner/repo

# Search
kdex search "authentication"

# Launch interactive mode
kdex
```

That's it. Your knowledge, instantly searchable.

---

## 🤖 AI Integration

kdex speaks [MCP](https://modelcontextprotocol.io/) (Model Context Protocol), so AI assistants can search your indexed content directly.

### GitHub Copilot CLI

Add to `~/.config/github-copilot/mcp.json`:

```json
{
  "mcpServers": {
    "kdex": { "command": "kdex", "args": ["mcp"] }
  }
}
```

### Claude Desktop

Add to `~/.config/claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "kdex": { "command": "kdex", "args": ["mcp"] }
  }
}
```

### Ollama (Local LLMs)

Pipe search results to your local model:

```bash
kdex search "error handling" --json | \
  jq -r '.results[].snippet' | \
  ollama run llama3 "Explain this code:"
```

See [MCP Integration Guide](doc/mcp-integration.md) for more details.

---

## 📖 What It Indexes

| Source | Examples |
|--------|----------|
| **Code Repositories** | Any git repo, monorepos, microservices |
| **Knowledge Bases** | Obsidian vaults, Logseq graphs, Dendron |
| **Documentation** | Markdown wikis, READMEs, technical docs |
| **Config Files** | YAML, TOML, JSON with searchable content |

kdex respects `.gitignore` and skips binary files automatically.

---

## 💻 Usage

### Interactive Mode (TUI)

```bash
kdex
```

Full-screen interface with:
- Real-time search as you type
- File preview panel (`Ctrl+P`)
- Repository management
- Keyboard-driven navigation

### Command Line

```bash
kdex index <path>              # Index a directory
kdex search <query>            # Search indexed content
kdex search <query> --json     # JSON output for scripting
kdex list                      # List indexed repositories
kdex remove <path>             # Remove from index
kdex mcp                       # Start MCP server
```

Run `kdex --help` for all options.

---

## 🔧 Installation

### From crates.io (recommended)

```bash
cargo install kdex
```

### From source

```bash
git clone https://github.com/urbanisierung/kdex.git
cd kdex
cargo install --path .
```

### Requirements

- Rust 1.88+ ([install via rustup](https://rustup.rs/))
- Works on Linux, macOS (Apple Silicon), and Windows

---

## 📚 Documentation

- [Features](doc/features.md) — Full feature list
- [Documentation](doc/documentation.md) — Detailed usage guide
- [MCP Integration](doc/mcp-integration.md) — AI assistant setup
- [Ollama Integration](doc/ollama-integration.md) — Local LLM workflow
- [Roadmap](doc/roadmap.md) — What's coming next

---

## 🛠️ Development

```bash
make ci        # Run full CI pipeline (Docker)
make ci-quick  # Quick checks (format + clippy)
make test      # Run tests
make build     # Build debug binary
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup.

---

## License

MIT © [urbanisierung](https://github.com/urbanisierung)
