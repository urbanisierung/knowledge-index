# Progress

## Changelog

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
