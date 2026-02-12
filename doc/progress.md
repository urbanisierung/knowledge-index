# Progress

## Changelog

### 2026-02-12

- **Fixed Cargo.lock tracking**
  - Removed Cargo.lock from .gitignore (should be tracked for binary applications)
  - Enables release workflow to commit version updates properly

### 2026-02-11

- **Fixed crates.io publish workflow**
  - Extract version from release tag (e.g., `v0.0.4` -> `0.0.4`)
  - Update Cargo.toml and Cargo.lock version before publishing
  - Commit version update back to main branch after successful publish
  - Use `--allow-dirty` flag since version is modified at publish time

- **Fixed CI formatting issues**
  - Applied Docker-based rustfmt to fix formatting across 10 files

- **Documentation & Landing Page Update**
  - Added Gemini CLI integration example to MCP documentation
  - Added full documentation section to landing page with all commands and options
  - Replaced external documentation links with in-page anchor links
  - Added Gemini CLI to AI integrations showcase

- **Implemented Phase 14 Features: Vault Presets & Search History**
  - Added VaultType enum with Obsidian, Logseq, Dendron, Generic variants
  - Auto-detect vault type when adding repositories (checks for `.obsidian/`, `logseq/`, `dendron.yml`)
  - Store vault type in database schema v5
  - Display vault type icons in `kdex list` output
  - Added SearchHistory with persistence to JSON (`~/.config/kdex/search_history.json`)
  - TUI search history navigation with Up/Down arrows when input is empty
  - Show history hint in empty search state

- **Implemented Phase 14 Features: Graph Visualization & Health Diagnostics**
  - Added `graph` command exporting DOT format for Graphviz visualization
  - Added `graph --json` for JSON output (web visualization tools)
  - Added `health` command with orphan detection and broken link checking
  - Health score (0-100) based on link quality and file connectivity
  - Added `get_all_links()`, `get_all_file_paths()`, `get_orphan_files()` methods

- **Implemented Phase 14 Features: Knowledge Statistics & Data Storage**
  - Added `stats` command showing file counts, tags, links, embeddings, and storage
  - Modified indexer to store tags and wiki-links in dedicated database tables
  - Added `add_tags()` and `add_links()` methods to database layer
  - Added `get_stats()` method returning comprehensive index statistics
  - Backlinks and tags commands now return actual data after re-indexing

- **Implemented Phase 14 Features: Search Enhancements & Developer Experience**
  - Added `completions` command for generating shell completions (bash, zsh, fish, PowerShell, elvish)
  - Added `backlinks` command for finding files that link to a target
  - Added `tags` command for listing all tags from indexed files
  - Added `context` command for building AI prompts from search results with token limits
  - Added `--fuzzy` flag for typo-tolerant search using Jaro-Winkler similarity
  - Added `--regex` flag for pattern matching with regular expressions
  - Added `--tag` flag for filtering by frontmatter tags (placeholder)
  - Extended database schema (v4) with dedicated tags and links tables
  - Added clap_complete, strsim, regex dependencies

- **Default Search Command**
  - Search is now the default command: `kdex "query"` works without typing `search`
  - Added short flags for search: `-r` (repo), `-t` (file-type), `-l` (limit), `-s` (semantic), `-H` (hybrid), `-g` (group-by-repo)
  - Argument rewriting detects if first arg is not a known command and treats it as a search query
  - Updated help text and documentation

### 2026-02-10

- **Implemented Phase 13: Remote Repositories & Configuration Portability**
  - Added `kdex add --remote owner/repo` command for cloning GitHub repos
  - Added `kdex sync` command for syncing remote repositories
  - Added `kdex config export` for exporting settings to YAML
  - Added `kdex config import` for importing settings with merge support
  - Remote repos cloned to `~/.config/kdex/repos/<owner>/<repo>/`
  - Support for SSH agent and GitHub token authentication
  - Shallow clone option for faster setup
  - Auto-delete cloned directories on `kdex remove`
  - Extended database schema (v3) for remote repo metadata
  - Added git2, serde_yaml, url dependencies

- **Added Phase 13: Remote Repositories & Configuration Portability**
  - Part 13.1: Remote GitHub repository support with auto-clone
  - Part 13.2: Background sync for keeping remote repos up-to-date
  - Part 13.3: Automatic cleanup when removing remote repos
  - Part 13.4: Configuration import/export for machine migration
  - Part 13.5: Optional GitHub API integration for repo discovery

- **Added GitHub Pages Workflow**
  - Created `.github/workflows/pages.yml` for deploying landing page
  - Deploys `docs/` folder to GitHub Pages on push to main

- **Completed Phase 12: Marketing & Growth**
  - Rewrote README with engaging hero section, feature tables, problem/solution format
  - Created `doc/ollama-integration.md` for local LLM workflows
  - Created `docs/index.html` - single-file landing page (9.5KB, dark mode, mobile-responsive)
  - Added all messaging pillars: local-first, AI-ready, instant setup, universal

- **Added Phase 12: Marketing & Growth to Roadmap**
  - README excellence with best practices from popular CLIs
  - Marketing message and positioning strategy
  - Ollama integration for local-first AI workflow
  - Single-file landing page design (< 20KB, loads in < 1s)

- **Renamed CLI to `kdex`** (Knowledge DEX)
  - Updated package name from `knowledge-index` to `kdex`
  - All CLI commands now use `kdex` instead of `knowledge-index`
  - Config directory auto-migrates from `~/.config/knowledge-index` to `~/.config/kdex`
  - Environment variable changed to `KDEX_CONFIG_DIR` (legacy still supported)
  - Updated all documentation, MCP server, and help text

- **Fixed x86_64-apple-darwin Build**
  - Removed x86_64-apple-darwin from release targets (Intel Mac legacy)
  - ort-sys doesn't provide prebuilt ONNX binaries for this target
  - Apple Silicon (aarch64-apple-darwin) still supported
  - Intel Mac users can build from source

- **Created Demo Script for asciinema**
  - Added `scripts/demo.sh` for recording terminal demos
  - Demonstrates indexing, searching, JSON output, MCP integration
  - Instructions for recording with asciinema

- **Search Efficiency Audit**
  - Verified FTS5 search works well for common queries
  - Tested "free port", "port 3000", tool names like "fuser"
  - Results include highlighted snippets with match markers
  - Search is efficient for AI assistant use cases

### 2026-02-09

- **Enhanced MCP Server Startup**
  - MCP server now prints helpful startup banner with available tools
  - Shows integration snippets for GitHub Copilot CLI and Claude Desktop
  - Displays config file paths for easy setup
  - Colorized output for better readability

- **Fixed TUI Keyboard Shortcuts**
  - Changed shortcuts to use Ctrl combinations to avoid conflicts with search input
  - `Ctrl+Q` - Quit (works while typing)
  - `Ctrl+P` - Toggle preview (works while typing)
  - `Ctrl+J/K` - Navigate up/down (works while typing)
  - `Ctrl+O` - Open file in editor
  - Arrow keys still work for navigation
  - Updated help overlay with new shortcuts

- **Improved MCP/Copilot Documentation**
  - Added GitHub Copilot CLI configuration example with file paths
  - Updated README.md with AI Integration section
  - Added `mcp` command to documentation.md
  - Removed "(coming soon)" from MCP description

- **Fixed CI Test Isolation Issues**
  - Added `KNOWLEDGE_INDEX_CONFIG_DIR` environment variable support for custom config directory
  - Updated all integration tests to use isolated temporary directories
  - Prevents test interference and CI environment differences from causing failures
  - Each test now runs with its own fresh database/config

- **Added Local CI Scripts (Makefile)**
  - Created `Makefile` with Docker-based CI commands
  - `make ci` runs full CI pipeline matching GitHub Actions
  - `make ci-quick` runs format and clippy checks only
  - `make ci-msrv` checks minimum supported Rust version (1.88)
  - Individual steps: `make ci-format`, `make ci-clippy`, `make ci-test`, `make ci-doc`
  - `make ci-test-verbose` for debugging test failures with output
  - Local development commands: `make build`, `make release`, `make test`, `make fmt`, `make lint`
  - Updated README.md with development section
  - Updated copilot-instructions.md with CI verification requirements

- **Fixed Integration Test Issues**
  - Improved test error messages to include stderr output for debugging
  - Added Windows `.exe` extension handling in binary path detection
  - Tests now show actual error message when commands fail

- **Fixed Additional CI Issues**
  - Updated MSRV to 1.88 (required by darling, ort-sys dependencies)
  - Fixed clippy warnings: `field_reassign_with_default`, `needless_raw_string_hashes`
  - Fixed `#[ignore]` without reason in integration tests
  - Removed `cargo test --doc` step (binary-only crate, no lib.rs)

### 2026-02-08

- **Fixed CI Pipeline Issues**
  - Updated MSRV from 1.75 to 1.85 (required for edition2024 in ort-sys dependency)
  - Fixed `cargo fmt` check syntax in CI workflow
  - Updated Cargo.toml rust-version to 1.85
  - Updated CI workflow MSRV job to use dtolnay/rust-toolchain@1.85.0

- **Implemented Phase 10: Polish and Release**
  - Cargo.toml Metadata
    - Added rust-version (MSRV 1.85)
    - Added homepage, documentation URLs
    - Added exclude patterns for packaging
    - Added text-processing category
  - Unit Tests
    - Config module tests (defaults, serialization, partial parsing)
    - Searcher module tests (FTS escaping, search modes)
    - 18 unit tests total, all passing
  - Integration Tests
    - CLI help and version tests
    - Config, list, search command tests
    - Full index/search cycle test (optional)
  - CI Pipeline (.github/workflows/ci.yml)
    - Multi-platform testing (Linux, macOS, Windows)
    - Rust stable and beta channels
    - MSRV check (1.85)
    - Format check, clippy lints, doc tests
    - Publish dry-run verification
  - Release Workflow (.github/workflows/release.yml)
    - Cross-platform binary builds
    - Archive creation (tar.gz, zip)
    - Checksum generation
    - GitHub release automation
    - crates.io publish on stable tags
  - MIT LICENSE file added
  - cargo publish --dry-run verified

- **Implemented Low Priority Optional Features**
  - Welcome Screen (First Run)
    - Detects when no repositories are indexed
    - Shows getting started guide
    - Press Enter to continue, q to quit
  - Debug Mode (`--debug` flag)
    - Enables `RUST_BACKTRACE=1` for detailed error traces
    - Shows hint to use --debug on errors
  - Shell Aliases
    - Added suggested aliases to --help output (ki, kis, kii)
  - Markdown Syntax Stripping
    - New config option `strip_markdown_syntax`
    - Strips headers, bold/italic, links, blockquotes
    - Preserves code block content
  - Code Block Indexing
    - New config option `index_code_blocks`
    - Extracts code blocks with language tags
    - Useful for searching by programming language
  - Extended Config Command
    - All new config options accessible via CLI
    - Added: enable_semantic_search, embedding_model, default_search_mode
    - Added: strip_markdown_syntax, index_code_blocks

- **Implemented Medium Priority Optional Features**
  - TUI Preview Pane
    - Toggle with 'p' key in Search view
    - Shows file content with line numbers
    - Scroll with j/k in preview mode
    - Horizontal split: 40% results, 60% preview
  - Loading State Overlay
    - Animated spinner during operations
    - Centered overlay with message
    - Loading state management in App
  - Delete Confirmation Dialog
    - Modal dialog before repository deletion
    - Press 'y' to confirm, 'n'/Esc to cancel
    - Confirmation action system in App
  - Platform Limits Check
    - Linux inotify watch limit detection
    - Warns before starting file watcher
    - Estimates directory count for watched repos
    - Instructions for increasing limits

- **Implemented Priority Optional Features**
  - Added `--group-by-repo` flag to search command
    - Groups search results by repository in both CLI and JSON output
    - Shows repository headers with result counts
  - Added YAML frontmatter parsing for markdown files
    - Extracts title, tags from frontmatter
    - Stores in `markdown_meta` table
  - Added heading extraction from markdown files
    - Parses ATX-style headings (# through ######)
    - Stores heading hierarchy in database
  - Added wiki-link extraction (`[[link]]` and `[[link|display]]`)
  - Added `rebuild-embeddings` command
    - Regenerates embeddings without full re-index
    - Supports `--repo` filter for specific repositories
    - Shows progress indicator during processing
  - Added `watch` command for file system monitoring
    - Watches indexed repositories for changes
    - Automatic re-indexing on file changes
  - Added progress indicator for embedding generation
    - Shows file count progress during rebuild

- **Roadmap Review and Cleanup**
  - Verified all previously implemented items in Phases 1-7 and marked as checked
  - Marked optional/future items with *(Optional)*, *(Future)*, or *(Pre-release)* tags
  - Terminal setup/teardown (Part 4.1) was already implemented - marked complete
  - Size warning message (Part 4.2) was already implemented - marked complete
  - Keyboard shortcuts in status bar (Part 4.12) already implemented - marked complete
  - Accessibility features (visual, motor, cognitive) already implemented - marked complete
  - Documentation items (README, doc/ files, inline docs) already complete - marked complete
  - Marked Phase 9 Remote items as *(Future)* since they are intentionally deferred
  - Marked Phase 10 Polish items as *(Pre-release)* or *(Optional)* appropriately

- **Implemented Phase 8: Vector Search (Semantic Search)**
  - Added `fastembed` crate (v5) for local embedding generation
  - Uses all-MiniLM-L6-v2 model (384 dimensions, ~22MB)
  - Created `src/core/embedder.rs` with:
    - `Embedder` struct with model loading
    - Text chunking (~512 tokens with 50 token overlap)
    - Batch embedding generation
    - Query embedding for search
  - Extended database schema (version 2):
    - Added `embeddings` table for vector storage
    - Migration support for existing databases
    - Embeddings stored as binary blobs (f32 little-endian)
  - Added vector search to `Database`:
    - `store_embeddings()` for batch storage
    - `vector_search()` with cosine similarity
  - Enhanced `Searcher` with multi-mode search:
    - `SearchMode`: Lexical, Semantic, Hybrid
    - `search_with_mode()` for mode selection
    - Hybrid search with Reciprocal Rank Fusion (RRF, k=60)
  - Added CLI search mode flags:
    - `--semantic`: Use vector/embedding search
    - `--hybrid`: Combined lexical + semantic with RRF
    - `--lexical`: Full-text search (default)
  - Updated MCP server search tool:
    - Added `mode` parameter ("lexical", "semantic", "hybrid")
    - Response includes effective search mode used
  - Added config options:
    - `enable_semantic_search`: Toggle embedding generation
    - `embedding_model`: Model name (default: all-MiniLM-L6-v2)
    - `default_search_mode`: Default mode for searches

### 2026-02-07

- **Updated Roadmap Tracking**
  - Added rule to `.github/copilot-instructions.md` for checking off completed action items
  - Reviewed and checked off all completed items in `doc/roadmap.md` for Phases 1-5

- **Implemented Phase 6: Background Watcher (Core)**
  - Created `src/core/watcher.rs` with `IndexWatcher` struct
  - Implemented file system watching using `notify` crate
  - Added debouncing (500ms) for collecting events before processing
  - Event processing: map notify events to `ChangeType` (Created, Modified, Deleted)
  - Filtering: ignore patterns, binary files, files outside indexed repos
  - Batching changes by repository
  - Note: TUI integration deferred to future iteration

- **Implemented Phase 7: AI Integration (MCP Server)**
  - Created `src/mcp/` module with MCP server implementation
  - Added `rmcp` crate for Model Context Protocol support
  - Implemented MCP tools:
    - `search`: Search indexed content with optional filters
    - `list_repos`: List all indexed repositories
    - `get_file`: Get full file content with optional truncation
    - `get_context`: Get lines of context around a specific line number
  - `knowledge-index mcp` command starts stdio-based MCP server
  - Structured JSON responses optimized for LLM consumption
  - Truncation support with hints for follow-up actions

### 2026-02-06

- Initial project scaffold created
- Added `.github/copilot-instructions.md`
- Added documentation structure (`doc/`)
- Added `.gitignore` for Rust projects
- Initialized Rust project with `cargo init`
- Configured `Cargo.toml` with metadata and clippy lints
- Created comprehensive roadmap (`doc/roadmap.md`) with 10 phases
- Enhanced roadmap with detailed implementation specs, code examples, and dependency graph
- Added product vision, user journeys, and success criteria (PM review)
- Added UX specifications: onboarding, empty states, error messages, accessibility (UX review)
- Expanded problem statement to include knowledge management (Obsidian, markdown notes)
- Added markdown-aware processing: frontmatter, wiki-links, headings extraction
- Added knowledge management features to brainstormed section
- **Implemented Phase 1: Foundation**
  - Created configuration system (`src/config/mod.rs`) with OS-aware paths
  - Created error types (`src/error.rs`) with thiserror
  - Created database layer (`src/db/mod.rs`) with SQLite + FTS5
  - Database schema with repositories, files, and contents tables
- **Implemented Phase 2: Core Indexing Engine**
  - Created indexer (`src/core/indexer.rs`) with parallel file processing
  - Binary file detection (extension + null byte check)
  - Incremental indexing with mtime/size comparison
  - Content hashing with blake3
- **Implemented Phase 3: Search System**
  - Created searcher (`src/core/searcher.rs`) with FTS5 query escaping
  - Search filters by repository and file type
  - Result snippets with match highlighting
- **Implemented Phase 4: TUI (App Mode)**
  - Created TUI app (`src/tui/`) with ratatui
  - Search view with live search
  - Repos view for managing indexed repositories
  - Help overlay with keyboard shortcuts
  - Minimum terminal size detection
- **Implemented Phase 5: CLI Mode**
  - Created CLI commands: index, search, update, list, remove, config
  - Progress bar for indexing operations
  - JSON output option for scripting
  - Color-coded output with owo-colors
- **Code Quality: Zero-Warning Build**
  - Updated `.github/copilot-instructions.md` with Build Requirements section
  - Fixed all clippy warnings across the codebase:
    - `src/db/mod.rs`: Fixed `trivially_copy_pass_by_ref`, `match_same_arms`, `too_many_arguments`
    - `src/core/indexer.rs`: Fixed `cast_possible_truncation`, `cast_possible_wrap`
    - `src/core/searcher.rs`: Removed no-op replace calls in `escape_fts_query`
    - Added `#[allow(...)]` attributes for intentional patterns (CLI args, command functions)
  - Project now builds with `cargo clippy -- -D warnings` and `cargo build --release` with zero warnings
