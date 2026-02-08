# Roadmap

This document outlines the implementation roadmap for `knowledge-index`, organized into phases, parts, and action items.

**Implementation Notes:**
- Complete phases 1-5 for MVP (minimum viable product)
- Phases 6-8 are enhancements, can be done in parallel after MVP
- Phases 9-10 are for production readiness
- Each action item should result in working, tested code before moving on

---

## Product Vision

### Problem Statement

Developers and knowledge workers accumulate valuable information across many sources:

**Code Repositories**
- Multiple projects with interconnected codebases
- Internal libraries, utilities, and patterns
- Code comments and inline documentation

**Knowledge Collections**
- Markdown notes (Obsidian vaults, Dendron, Foam, Logseq)
- Personal wikis and documentation
- Research notes, bookmarks, and clippings
- Meeting notes and decision records

**The Challenge**

When working with AI assistants, users struggle to provide relevant context from these scattered sources. Current solutions fall short:

| Approach | Problem |
|----------|---------|
| Cloud-based search | Privacy concerns, subscription costs, vendor lock-in |
| Vector databases | Complex setup, requires embeddings infrastructure |
| Built-in AI context | Limited to open files, no cross-repo search |
| Manual copy-paste | Tedious, incomplete, doesn't scale |

**The Result:** AI assistants give generic answers because they lack access to the user's actual knowledge and code.

### Solution

A local-first CLI tool that creates a unified, searchable index across all your repositories and knowledge bases:

1. **Indexes everything** — code, markdown, documentation, notes
2. **Searches instantly** — full-text search with relevance ranking
3. **Integrates with AI** — MCP protocol for seamless assistant access
4. **Stays current** — automatic background re-indexing on changes
5. **Respects privacy** — 100% local, zero cloud dependencies

### Target Users

| User Type | Primary Use Case | Key Requirements |
|-----------|-----------------|------------------|
| Solo Developer | Search own projects and notes | Fast setup, low overhead |
| AI-assisted Developer | Provide context to Copilot/Claude | MCP integration, structured output |
| Knowledge Worker | Search Obsidian/markdown vaults | Markdown-aware, link support |
| Team Lead | Onboard to new codebases | Multi-repo search, good UX |
| Researcher | Find across collected materials | Large corpus support, fast search |

### Success Criteria

- First index created within 2 minutes of install
- Search results appear in <100ms
- MCP server connects to AI tool without configuration errors
- User can accomplish core tasks without reading documentation
- Markdown content (headings, links, code blocks) searchable and well-formatted

---

## User Journeys

### Journey 1: First-Time Setup (Developer)

```
1. User installs: cargo install knowledge-index
2. User navigates to project: cd ~/projects/my-app
3. User indexes: knowledge-index index
   → Progress bar shows indexing
   → "Indexed 1,234 files in 3.2s"
4. User searches: knowledge-index search "database connection"
   → Results displayed with file paths and snippets
```

**Success:** User finds relevant code in under 5 minutes from install.

### Journey 2: Indexing Knowledge Vault (Knowledge Worker)

```
1. User has Obsidian vault: ~/Documents/notes
2. User indexes: knowledge-index index ~/Documents/notes
   → "Indexed 847 markdown files in 1.8s"
3. User indexes code too: knowledge-index index ~/projects/api
4. User searches across both: knowledge-index search "authentication flow"
   → Results from notes AND code, ranked by relevance
```

**Success:** User searches across notes and code in one query.

### Journey 3: Daily Development with AI

```
1. User configures MCP in Claude Desktop (one-time)
2. User asks Claude: "How does authentication work in my projects?"
3. Claude calls knowledge-index MCP tools
4. Claude provides answer with specific file references
```

**Success:** AI gives accurate, context-aware answers.

### Journey 4: Research with AI Context

```
1. User has indexed: research notes, API docs, code samples
2. User asks AI: "Based on my notes, what approach should I use for caching?"
3. AI searches index, finds relevant notes and code examples
4. AI provides recommendation citing user's own collected knowledge
```

**Success:** AI leverages user's personal knowledge base for tailored advice.

### Journey 5: Exploring in TUI

```
1. User launches: knowledge-index
2. TUI opens with search focused
3. User types query, results appear live
4. User navigates with arrows, presses Enter to preview
5. User presses 'o' to open file in editor
```

**Success:** User finds and opens file without leaving terminal.

---

## Quick Reference

### Phase Dependencies

```
Phase 1 (Foundation)
    ↓
Phase 2 (Indexing) ──→ Phase 6 (Watcher) [optional]
    ↓                        ↓
Phase 3 (Search) ←───────────┘
    ↓
Phase 4 (TUI) + Phase 5 (CLI) [parallel]
    ↓
Phase 7 (MCP) ──→ Phase 8 (Vector) [optional]
    ↓
Phase 9 (Remote) [future] + Phase 10 (Release)
```

### MVP Checklist (Phases 1-5)

- [x] Config system with OS-aware paths
- [x] SQLite database with FTS5
- [x] `index` command indexes a directory
- [x] `search` command finds content
- [x] `update` command refreshes index
- [x] `list` command shows repositories
- [x] `remove` command deletes from index
- [x] TUI with search and repo management
- [x] All commands work with `--help`, `--json`, `--quiet`

### Definition of Done (per action item)

1. Code compiles without warnings (`cargo clippy`)
2. Code is formatted (`cargo fmt`)
3. Basic tests pass (if applicable)
4. `doc/progress.md` updated
5. Feature documented in `doc/documentation.md` (if user-facing)

---

## Phase 1: Foundation

Establish core infrastructure, configuration, and project structure.

**Goal:** Project compiles, runs, and has basic scaffolding in place.

### Part 1.1: Configuration System

- [x] Create `src/config/mod.rs` with compile-time constants:
  ```rust
  pub const APP_NAME: &str = "knowledge-index";
  pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
  pub const CONFIG_FILE_NAME: &str = "config.toml";
  pub const DATABASE_FILE_NAME: &str = "index.db";
  ```
- [x] Implement OS-aware config directory detection using `dirs` crate:
  - Linux: `~/.config/knowledge-index/`
  - macOS: `~/Library/Application Support/knowledge-index/`
  - Windows: `%APPDATA%\knowledge-index\`
- [x] Define `Config` struct with serde for TOML parsing:
  ```rust
  #[derive(Debug, Deserialize, Serialize)]
  pub struct Config {
      pub max_file_size_mb: u32,        // Skip files larger than this (default: 10)
      pub ignore_patterns: Vec<String>, // Additional patterns to ignore
      pub default_language: Option<String>,
      pub color_enabled: bool,
  }
  ```
- [x] Implement config loading: check file exists → load → merge with defaults → validate
- [x] Create config directory and default file on first run if not exists
- [x] Add `Config::default()` with sensible values

### Part 1.2: Project Structure

- [x] Set up module structure:
  ```
  src/
    main.rs           # Entry point, minimal logic
    lib.rs            # Re-exports public API
    cli/
      mod.rs          # CLI module root
      args.rs         # Clap argument definitions
      commands/       # One file per subcommand
        mod.rs
        index.rs
        search.rs
        update.rs
        list.rs
        remove.rs
        config.rs
    tui/
      mod.rs          # TUI module root
      app.rs          # Application state
      ui.rs           # Widget rendering
      event.rs        # Event handling
      views/          # Different screens
        mod.rs
        search.rs
        repos.rs
        help.rs
    core/
      mod.rs
      indexer.rs      # Indexing logic
      searcher.rs     # Search logic
      watcher.rs      # File watching (Phase 6)
    db/
      mod.rs
      schema.rs       # Table definitions
      queries.rs      # Prepared statements
      migrations.rs   # Schema versioning
    config/
      mod.rs
    error.rs          # Custom error types with thiserror
  ```
- [x] Add `clap` with derive feature for CLI parsing
- [x] Define root `Cli` struct with subcommands enum
- [x] Implement dual-mode detection logic:
  ```rust
  // If no subcommand AND stdin is TTY → launch TUI
  // If subcommand provided → execute CLI command
  // If stdin not TTY → CLI mode only (for pipes)
  ```

### Part 1.3: Database Layer

- [x] Add `rusqlite` with `bundled` and `modern_sqlite` features (includes FTS5)
- [x] Design complete database schema:
  ```sql
  -- Schema version tracking
  CREATE TABLE IF NOT EXISTS schema_version (
      version INTEGER PRIMARY KEY
  );
  
  -- Indexed repositories
  CREATE TABLE IF NOT EXISTS repositories (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      path TEXT NOT NULL UNIQUE,          -- Absolute canonical path
      name TEXT NOT NULL,                  -- Directory name for display
      created_at TEXT NOT NULL,            -- ISO 8601 timestamp
      last_indexed_at TEXT,                -- NULL if never completed
      file_count INTEGER DEFAULT 0,
      total_size_bytes INTEGER DEFAULT 0,
      status TEXT DEFAULT 'pending'        -- pending, indexing, ready, error
  );
  
  -- Individual files
  CREATE TABLE IF NOT EXISTS files (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      repo_id INTEGER NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
      relative_path TEXT NOT NULL,         -- Path relative to repo root
      content_hash TEXT NOT NULL,          -- BLAKE3 hash for change detection
      file_size_bytes INTEGER NOT NULL,
      last_modified_at TEXT NOT NULL,      -- File mtime as ISO 8601
      language TEXT,                       -- Detected language (rust, python, etc.)
      UNIQUE(repo_id, relative_path)
  );
  
  -- Full-text search content
  CREATE VIRTUAL TABLE IF NOT EXISTS contents USING fts5(
      file_id UNINDEXED,                   -- Foreign key to files
      content,                             -- File content for searching
      tokenize='porter unicode61'          -- Stemming + unicode support
  );
  
  -- Index for common queries
  CREATE INDEX IF NOT EXISTS idx_files_repo ON files(repo_id);
  CREATE INDEX IF NOT EXISTS idx_files_hash ON files(content_hash);
  ```
- [x] Implement `Database` struct with connection management
- [x] Add migration system: check version → apply pending migrations in order
- [x] Create `DatabaseError` enum with descriptive variants
- [x] **Note:** SQLite doesn't need connection pooling for single-process; use `Mutex<Connection>` for thread safety

---

## Phase 2: Core Indexing Engine

Build the high-performance file indexing system.

**Goal:** `knowledge-index index` works and populates the database correctly.

### Part 2.1: File Discovery

- [x] Use `ignore` crate (not `walkdir`) — it respects `.gitignore` automatically
- [x] Configure walker with parallel iteration enabled
- [x] Create `FileFilter` that checks:
  - File extension against known binary types (`.exe`, `.bin`, `.png`, `.jpg`, etc.)
  - File size against `config.max_file_size_mb`
  - Magic bytes for binary detection (first 8KB, check for null bytes)
- [x] Implement file type detection by extension mapping:
  ```rust
  fn detect_file_type(path: &Path) -> FileType {
      match path.extension()?.to_str()? {
          // Code
          "rs" => FileType::Code("rust"),
          "py" => FileType::Code("python"),
          "js" | "jsx" => FileType::Code("javascript"),
          "ts" | "tsx" => FileType::Code("typescript"),
          // Knowledge/Documentation
          "md" | "markdown" => FileType::Markdown,
          "txt" => FileType::PlainText,
          "org" => FileType::OrgMode,
          "rst" => FileType::ReStructuredText,
          // Config
          "json" | "yaml" | "yml" | "toml" => FileType::Config,
          _ => FileType::Unknown
      }
  }
  ```
- [x] Support custom ignore patterns from config (glob syntax)
- [x] Respect `.obsidian/` and similar tool directories (ignore by default)
- [x] Log skipped files at debug level for troubleshooting

### Part 2.2: Content Processing

- [x] Use `rayon::par_iter()` on collected file paths for parallel processing
- [x] Read files with size limit check before full read:
  ```rust
  // Read up to max_size + 1 byte to detect oversized files
  let mut buffer = Vec::with_capacity(max_size + 1);
  file.take(max_size as u64 + 1).read_to_end(&mut buffer)?;
  if buffer.len() > max_size { return Err(FileTooLarge); }
  ```
- [x] Compute BLAKE3 hash of content (fast, parallelized internally)
- [x] Detect encoding using `encoding_rs` or BOM detection:
  - Try UTF-8 first (most common)
  - Fall back to Latin-1 for legacy files
  - Skip files that can't be decoded as text
- [x] Normalize line endings (CRLF → LF) before storing
- [x] Strip excessive whitespace but preserve structure for search context

### Part 2.3: Markdown-Aware Processing

For markdown files (`.md`), extract additional metadata:

**Note:** These are optional enhancements for Phase 2. The database schema for `markdown_meta` exists but is not populated. Markdown files are still indexed and searchable via full-text search.

- [x] Parse YAML frontmatter (common in Obsidian, Hugo, Jekyll):
  ```yaml
  ---
  title: My Note
  tags: [rust, programming]
  created: 2024-01-15
  ---
  ```
- [x] Extract headings for better search context
- [x] Detect and index wiki-style links: `[[Other Note]]`, `[[note|display text]]`
- [x] Index code blocks with their language tags (config option)
- [x] Store metadata in separate table for filtered searches (schema exists):
  ```sql
  CREATE TABLE IF NOT EXISTS markdown_meta (
      file_id INTEGER PRIMARY KEY REFERENCES files(id),
      title TEXT,
      tags TEXT,           -- JSON array
      links TEXT,          -- JSON array of [[links]]
      headings TEXT        -- JSON array of headings
  );
  ```
- [x] Config option to strip markdown syntax for cleaner full-text search
- [ ] *(Optional)* Preserve original for accurate snippets

### Part 2.4: Index Management

- [x] Implement `index` command flow:
  1. Resolve path to absolute canonical form
  2. Check if already indexed → prompt for update or skip
  3. Insert repository record with status='indexing'
  4. Walk and process files, collecting results
  5. Batch insert into `files` and `contents` tables (100 files per transaction)
  6. Update repository status='ready' with final counts
- [x] Create progress reporting struct:
  ```rust
  struct IndexProgress {
      total_files: usize,
      processed_files: usize,
      skipped_files: usize,
      current_file: String,
      bytes_processed: u64,
      start_time: Instant,
  }
  ```
- [x] Use `indicatif` crate for progress bar in CLI mode
- [x] Implement `update` command:
  - Single repo: `knowledge-index update /path/to/repo`
  - All repos: `knowledge-index update --all`
  - Use incremental indexing (Part 2.4)

### Part 2.4: Incremental Indexing

- [x] For `update` command, implement smart diff:
  1. Load existing file records for repository
  2. Walk current files, compare `(relative_path, mtime, size)` tuples
  3. Categorize into: unchanged, modified, new, deleted
  4. Skip unchanged files entirely
  5. Re-process modified and new files
  6. Delete removed files from index
- [x] Compare content hash only if mtime differs (avoid re-reading unchanged files)
- [x] Track and report: "Updated X files, added Y, removed Z, unchanged W"
- [x] Add `--force` flag to skip smart detection and re-index everything
- [x] Handle edge case: file exists in DB but is now unreadable (permission denied)

---

## Phase 3: Search System

Implement fast and relevant search capabilities.

**Goal:** `knowledge-index search "query"` returns ranked, formatted results.

### Part 3.1: Full-Text Search

- [x] Build FTS5 query wrapper that handles:
  - Escaping special characters (`"`, `*`, etc.)
  - Converting user query to FTS5 syntax
  - Supporting quoted phrases: `"exact match"`
  - Supporting prefix: `func*`
  - Supporting boolean: `rust AND async`, `error OR warning`
- [x] Use BM25 ranking (built into FTS5):
  ```sql
  SELECT f.relative_path, r.path as repo_path, 
         snippet(contents, 1, '>>>', '<<<', '...', 64) as snippet,
         bm25(contents) as rank
  FROM contents c
  JOIN files f ON c.file_id = f.id
  JOIN repositories r ON f.repo_id = r.id
  WHERE contents MATCH ?
  ORDER BY rank
  LIMIT ? OFFSET ?
  ```
- [x] Implement snippet extraction with configurable context size
- [x] Add highlight markers that work for both terminal (ANSI) and plain text
- [x] Handle empty results gracefully with helpful message

### Part 3.2: Search Filters

- [x] Add filter flags to search command:
  - `--repo <name>` — filter by repository name (substring match)
  - `--lang <language>` — filter by detected language
  - `--path <pattern>` — filter by file path (glob pattern)
  - `--ext <extension>` — filter by file extension
  - `--type <type>` — filter by content type: `code`, `markdown`, `config`, `all`
  - `--tag <tag>` — filter markdown files by frontmatter tag
- [x] Implement filter application in SQL WHERE clause
- [x] Allow combining multiple filters (AND logic)
- [x] Default `--type all` searches everything; users can narrow down

### Part 3.3: Search Output

- [x] Define `SearchResult` struct:
  ```rust
  pub struct SearchResult {
      pub repo_name: String,
      pub repo_path: PathBuf,
      pub file_path: PathBuf,      // Relative to repo
      pub absolute_path: PathBuf,  // Full path for opening
      pub snippet: String,
      pub line_number: Option<u32>,
      pub file_type: FileType,     // Code, Markdown, Config, etc.
      pub language: Option<String>,
      pub score: f64,
      // Markdown-specific (optional)
      pub title: Option<String>,   // From frontmatter
      pub tags: Option<Vec<String>>,
  }
  ```
- [x] CLI output format (default):
  ```
  repo-name:src/main.rs:42
    ...matched >>>content<<< here...
  ```
- [x] JSON output format (`--json`):
  ```json
  {"results": [...], "total": 42, "query": "...", "took_ms": 12}
  ```
- [x] Add `--limit` (default: 20) and `--offset` for pagination
- [x] Add `--count` flag to only return total count
- [x] Implement `--group-by-repo` to cluster results by repository

---

## Phase 4: Terminal User Interface (App Mode)

Build the interactive TUI using `ratatui` with `crossterm` backend.

**Goal:** Running `knowledge-index` without arguments opens a full-screen interactive interface.

### Part 4.1: TUI Foundation

- [x] Add dependencies: `ratatui`, `crossterm`
- [x] Create `App` struct to hold all application state:
  ```rust
  pub struct App {
      pub mode: AppMode,           // Search, RepoList, Help
      pub search_input: String,
      pub search_results: Vec<SearchResult>,
      pub selected_index: usize,
      pub repos: Vec<Repository>,
      pub status_message: Option<(String, StatusLevel)>,
      pub should_quit: bool,
  }
  ```
- [x] Implement terminal setup/teardown:
  ```rust
  fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
      enable_raw_mode()?;
      execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)?;
      // ...
  }
  
  fn restore_terminal(terminal: &mut Terminal<...>) -> Result<()> {
      disable_raw_mode()?;
      execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
      // ...
  }
  ```
- [x] Use `std::panic::set_hook` to restore terminal on panic
- [x] Create main event loop:
  ```rust
  loop {
      terminal.draw(|f| ui::render(f, &app))?;
      if let Event::Key(key) = event::read()? {
          handle_key_event(&mut app, key)?;
      }
      if app.should_quit { break; }
  }
  ```
- [x] Handle `Ctrl+C` and `Ctrl+D` via key events (don't let them crash)

### Part 4.2: Minimum Terminal Size

- [x] Define constants: `MIN_WIDTH = 60`, `MIN_HEIGHT = 15`
- [x] Check size in render function before drawing:
  ```rust
  fn render(frame: &mut Frame, app: &App) {
      let size = frame.area();
      if size.width < MIN_WIDTH || size.height < MIN_HEIGHT {
          render_size_warning(frame, size);
          return;
      }
      // Normal rendering...
  }
  ```
- [x] Size warning message:
  ```
  Terminal too small!
  Current: 40x10
  Required: 60x15
  Please resize your terminal.
  ```
- [x] Subscribe to resize events and trigger re-render

### Part 4.3: Layout Structure

- [x] Define main layout (vertical split):
  ```
  ┌─────────────────────────────────┐
  │ Header: Mode tabs / Title       │  3 rows
  ├─────────────────────────────────┤
  │                                 │
  │ Main Content Area               │  remaining
  │ (Search / Repos / Help)         │
  │                                 │
  ├─────────────────────────────────┤
  │ Status Bar: Keys / Messages     │  1 row
  └─────────────────────────────────┘
  ```
- [x] Use `Layout::vertical()` with constraints
- [x] Make content area adapt to available space

### Part 4.4: Search View

- [x] Search input widget at top of content area
- [x] Debounce search queries (300ms delay after last keystroke)
- [x] Results list with selectable items:
  - Show file path, repo name, snippet preview
  - Highlight selected row
  - Arrow keys to navigate, Enter to preview full file
- [x] *(Optional)* Preview pane (horizontal split when result selected):
  - Show file content with search term highlighted
  - Scroll with j/k or arrow keys
  - Show line numbers
- [x] Keybindings:
  - `/` or start typing → focus search input
  - `Esc` → clear search / go back
  - `Enter` → preview file
  - `o` → open file in `$EDITOR`
  - `y` → copy file path to clipboard

### Part 4.5: Repository Management View

- [x] List all indexed repositories as selectable list:
  ```
  ● my-project         │ 1,234 files │ 2 hours ago
  ○ another-repo       │   456 files │ 1 day ago
  ! broken-repo        │     - files │ error: not found
  ```
- [x] Status indicators: ● ready, ○ pending, ! error
- [x] Actions on selected repo:
  - `u` → update/re-index
  - `d` → delete from index (with confirmation)
  - `Enter` → show files in this repo
  - `o` → open directory in file manager
- [x] Add new repo: `a` → prompt for path (or use file picker)

### Part 4.6: Help Overlay

- [x] Toggle with `?` key from any view
- [x] Modal overlay (semi-transparent background)
- [x] List all keybindings organized by category
- [x] Dismiss with `?`, `Esc`, or `q`

### Part 4.7: Visual Design

- [x] Define color palette struct for consistency:
  ```rust
  pub struct Theme {
      pub bg: Color,
      pub fg: Color,
      pub accent: Color,          // Selected items, highlights
      pub muted: Color,           // Secondary text
      pub error: Color,
      pub warning: Color,
      pub success: Color,
  }
  ```
- [x] Use default terminal colors where possible (respect user themes)
- [x] Status bar format: `[Mode] | message | key hints`
- [x] Add `--no-color` flag to disable colors in TUI (accessibility)

### Part 4.8: First-Run Experience (Onboarding)

**Note:** These are optional UX enhancements. The TUI works without them.

- [x] Detect first run (no repositories indexed)
- [x] Show welcome screen in TUI with getting started guide
- [x] Auto-dismiss after user presses Enter or indexes first repo
- [x] Don't show again once user has indexed at least one repo

### Part 4.9: Empty States

Define helpful messages when no data exists:

**Note:** Basic empty state handling exists. Enhanced messages are optional.

- [ ] *(Optional)* No repositories indexed:
  ```
  No repositories indexed yet.
  
  Press 'a' to add a repository, or run:
    knowledge-index index /path/to/project
  ```
- [x] Search with no results (basic implementation exists):
  ```
  No results for "your query"
  
  Tips:
  • Try fewer or different keywords
  • Check spelling
  • Use prefix matching: "func*"
  ```
- [ ] *(Optional)* Repository list empty:
  ```
  No indexed repositories.
  Press 'a' to add one.
  ```

### Part 4.10: Loading States & Feedback

**Note:** Progress bars are shown during CLI indexing. TUI spinners are optional enhancements.

- [x] *(Optional)* Show spinner during operations:
  - Searching (if >100ms)
  - Indexing
  - Updating
  - Deleting
- [ ] *(Optional)* Progress indication for long operations:
  ```
  Indexing my-project...
  [████████░░░░░░░░] 45% (1,234 / 2,741 files)
  ```
- [x] Success feedback (exists via status messages):
  ```
  ✓ Indexed 2,741 files in 4.2s
  ```
- [x] Error feedback (exists via status messages):
  ```
  ✗ Failed to index: Permission denied
    /path/to/restricted/folder
  [Press Enter to dismiss]
  ```

### Part 4.11: Confirmation Dialogs

**Note:** Delete confirmation exists in CLI (`--force` flag). TUI modal dialogs are optional.

- [x] *(Optional)* Delete repository confirmation:
  ```
  ┌─────────────────────────────────────────┐
  │  Remove "my-project" from index?        │
  │                                         │
  │  This will delete the index data.       │
  │  The actual files won't be affected.    │
  │                                         │
  │  [Y]es    [N]o                          │
  └─────────────────────────────────────────┘
  ```
- [x] *(Optional)* Use consistent pattern: show options, highlight default
- [x] *(Optional)* Support both lowercase and uppercase for confirmation keys

### Part 4.12: Keyboard Navigation Principles

Document consistent navigation patterns:

| Context | Key | Action |
|---------|-----|--------|
| Lists | `↑`/`k` | Move up |
| Lists | `↓`/`j` | Move down |
| Lists | `Enter` | Select/Confirm |
| Lists | `Esc` | Cancel/Back |
| Global | `q` | Quit application |
| Global | `?` | Toggle help |
| Global | `Tab` | Switch between panes |
| Global | `Ctrl+C` | Force quit |
| Input | `Ctrl+U` | Clear input |
| Input | `Ctrl+W` | Delete word |

- [x] Display relevant shortcuts in status bar based on current context
- [x] Never require mouse - all actions accessible via keyboard

---

## Phase 5: CLI Mode

Complete the traditional command-line interface.

**Goal:** All commands work correctly with proper help, error handling, and output formats.

### Part 5.1: Command Structure

Define all commands with clap derive:

```rust
#[derive(Parser)]
#[command(name = "knowledge-index", about = "Index repositories for AI-powered search")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
    
    #[arg(long, global = true)]
    pub json: bool,
    
    #[arg(long, global = true)]
    pub quiet: bool,
    
    #[arg(long, global = true)]
    pub no_color: bool,
    
    #[arg(long, short, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Index a directory
    Index {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        force: bool,
    },
    /// Search indexed content
    Search {
        query: String,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        lang: Option<String>,
        #[arg(long, default_value = "20")]
        limit: usize,
        #[arg(long, default_value = "0")]
        offset: usize,
    },
    /// Update existing index
    Update {
        path: Option<PathBuf>,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        force: bool,
    },
    /// List indexed repositories
    List,
    /// Remove repository from index
    Remove {
        path: PathBuf,
        #[arg(long, short)]
        yes: bool,  // Skip confirmation
    },
    /// Show or edit configuration
    Config {
        #[arg(long)]
        edit: bool,
        #[arg(long)]
        path: bool,  // Just print config path
    },
    /// Start MCP server (Phase 7)
    Mcp,
}
```

- [x] Implement each command handler in separate file under `cli/commands/`
- [x] Entry point logic: `None` command → check TTY → launch TUI or show help

### Part 5.2: Help System

- [x] Add `#[command(after_help = "...")]` with examples for each command:
  ```
  Examples:
    knowledge-index index ~/projects/my-app
    knowledge-index search "async fn" --lang rust
    knowledge-index update --all
  ```
- [x] Include version info: `#[command(version, long_version = build_info())]`
- [x] Add `--help` examples that show common workflows
- [ ] *(Optional)* Consider adding `knowledge-index help <topic>` for detailed guides

### Part 5.3: Output Formats

- [x] Create `OutputFormat` enum and detection:
  ```rust
  fn detect_output_format(cli: &Cli) -> OutputFormat {
      if cli.json { return OutputFormat::Json; }
      if cli.quiet { return OutputFormat::Quiet; }
      if cli.no_color || !std::io::stdout().is_terminal() {
          return OutputFormat::Plain;
      }
      OutputFormat::Pretty
  }
  ```
- [ ] *(Optional)* Implement `Printable` trait for consistent output:
  ```rust
  trait Printable {
      fn print_pretty(&self, colors: bool);
      fn print_json(&self);
      fn print_quiet(&self);
  }
  ```
- [x] Use `owo-colors` for colored output with terminal detection
- [x] Ensure all commands respect `--quiet` (only errors to stderr)

### Part 5.4: Error Handling

- [x] Define application error type with `thiserror`:
  ```rust
  #[derive(Error, Debug)]
  pub enum AppError {
      #[error("Repository not found: {0}")]
      RepoNotFound(PathBuf),
      #[error("Database error: {0}")]
      Database(#[from] rusqlite::Error),
      #[error("Path does not exist: {0}")]
      PathNotFound(PathBuf),
      #[error("Not a directory: {0}")]
      NotADirectory(PathBuf),
      // ...
  }
  ```
- [x] Print errors to stderr with context
- [x] Use exit codes: 0 = success, 1 = error, 2 = usage error
- [ ] *(Optional)* Show suggestion when possible ("Did you mean...?")

### Part 5.5: CLI User Experience

#### Friendly Error Messages

Avoid cryptic errors. Always include:
1. What went wrong
2. Why it might have happened
3. How to fix it

```
# Bad
Error: ENOENT

# Good
Error: Repository not found: /home/user/old-project

The path doesn't exist or has been moved.

To see indexed repositories: knowledge-index list
To remove stale entries: knowledge-index remove /home/user/old-project
```

#### Command Feedback

- [x] Confirm destructive actions:
  ```
  $ knowledge-index remove my-project
  Remove "my-project" from index? [y/N] 
  ```
- [x] Show progress for long operations:
  ```
  $ knowledge-index index ~/large-monorepo
  Scanning files... 12,345 found
  Indexing [████████████░░░░░░░░] 60% (7,407/12,345)
  ```
- [x] Summarize results:
  ```
  $ knowledge-index update --all
  Updated 3 repositories:
    ✓ my-app         +12 files, -3 files, ~45 files unchanged
    ✓ api-service    no changes
    ✗ old-project    path not found (use --prune to remove)
  ```

#### Helpful Hints

- [x] When no repos indexed:
  ```
  $ knowledge-index search "test"
  No repositories indexed yet.
  
  Get started by indexing a project:
    knowledge-index index /path/to/project
  ```
- [x] When search returns no results:
  ```
  $ knowledge-index search "xyznonexistent"
  No results found for "xyznonexistent"
  
  Suggestions:
    • Check spelling
    • Try broader search terms
    • Use --repo to search specific repository
  ```
- [x] After first successful index:
  ```
  ✓ Indexed 1,234 files in 3.2s

  What's next:
    • Search: knowledge-index search "your query"
    • Browse: knowledge-index (opens TUI)
    • AI integration: knowledge-index mcp --help
  ```

#### Scripting-Friendly Mode

- [x] `--quiet` suppresses all non-error output
- [x] `--json` outputs machine-readable format
- [x] Exit codes are consistent and documented
- [x] No interactive prompts when stdin is not TTY

---

## Phase 6: Background Watcher (Auto Re-indexing)

Implement filesystem monitoring for automatic index updates.

**Goal:** Index stays fresh automatically during TUI sessions.

### Part 6.1: File Watcher Setup

- [x] Add `notify` crate with `macos_fsevent` feature for better macOS support
- [x] Create `Watcher` struct:
  ```rust
  pub struct IndexWatcher {
      watcher: RecommendedWatcher,
      watched_paths: HashSet<PathBuf>,
      pending_changes: Arc<Mutex<HashMap<PathBuf, ChangeType>>>,
      debounce_duration: Duration,
  }
  
  enum ChangeType {
      Created,
      Modified,
      Deleted,
  }
  ```
- [x] Implement debouncing: collect events for 500ms before processing
- [x] Handle platform limits:
  - Linux: check `max_user_watches`, log warning if too low
  - macOS: FSEvents has no practical limit
  - Windows: ReadDirectoryChangesW works per-directory

### Part 6.2: Event Processing

- [x] Map notify events to our change types:
  ```rust
  fn handle_event(event: notify::Event) -> Option<(PathBuf, ChangeType)> {
      match event.kind {
          EventKind::Create(_) => Some((path, ChangeType::Created)),
          EventKind::Modify(ModifyKind::Data(_)) => Some((path, ChangeType::Modified)),
          EventKind::Remove(_) => Some((path, ChangeType::Deleted)),
          _ => None,  // Ignore metadata changes, renames (handled as delete+create)
      }
  }
  ```
- [x] Filter events:
  - Ignore paths matching ignore patterns
  - Ignore binary files
  - Ignore files outside indexed repositories
- [x] Batch changes by repository for efficient updates

### Part 6.3: Background Thread Integration

**Note:** File watcher exists as a standalone background process (`knowledge-index watch`). TUI integration is optional.

- [ ] *(Optional)* Spawn watcher thread when TUI starts:
  ```rust
  let (tx, rx) = mpsc::channel();
  let watcher_handle = thread::spawn(move || {
      run_watcher(watched_repos, tx);
  });
  ```
- [ ] *(Optional)* Process pending changes in TUI event loop during idle:
  ```rust
  // In main loop, check for pending changes every N iterations
  if let Ok(changes) = rx.try_recv() {
      app.pending_reindex.extend(changes);
      app.status_message = Some("Changes detected, re-indexing...");
  }
  ```
- [ ] *(Optional)* Perform incremental re-index for batched changes
- [ ] *(Optional)* Update status bar: "Re-indexed 5 files" or spinner during active re-index
- [ ] *(Optional)* Handle watcher errors gracefully (don't crash TUI)

### Part 6.4: Standalone Daemon (Future)

**Note:** Basic watcher functionality exists via `knowledge-index watch --all`. Full daemon mode is a future enhancement.

- [ ] *(Future)* Add `knowledge-index daemon` command (future implementation)
- [ ] *(Future)* Create PID file to prevent multiple instances
- [ ] *(Future)* Log to file in config directory
- [ ] *(Future)* Provide systemd unit file template in docs
- [ ] *(Future)* IPC via Unix socket for status queries

---

## Phase 7: AI Integration

Enable seamless integration with AI assistants.

**Goal:** `knowledge-index mcp` starts a server that AI tools can connect to.

### Part 7.1: MCP Server Implementation

- [x] Add dependencies: `rmcp`, `tokio` (for async), `serde_json`
- [x] Create MCP server struct:
  ```rust
  #[derive(Clone)]
  pub struct KnowledgeIndexMcp {
      db: Arc<Mutex<Database>>,
      config: Arc<Config>,
      tool_router: ToolRouter<Self>,
  }
  ```
- [x] Implement stdio transport (for local AI tools):
  ```rust
  #[tokio::main]
  async fn run_mcp_server(db: Database, config: Config) -> Result<()> {
      let service = KnowledgeIndexMcp::new(db, config)
          .serve(stdio())
          .await?;
      service.waiting().await?;
      Ok(())
  }
  ```
- [x] **Important:** Never write to stdout except MCP protocol messages; use stderr for logs

### Part 7.2: MCP Tools

Expose these tools via MCP:

```rust
#[tool(description = "Search indexed code repositories")]
async fn search(
    &self,
    #[arg(description = "Search query")] query: String,
    #[arg(description = "Max results to return")] limit: Option<u32>,
    #[arg(description = "Filter by repository name")] repo: Option<String>,
    #[arg(description = "Filter by language")] lang: Option<String>,
) -> Result<CallToolResult, McpError>;

#[tool(description = "List all indexed repositories")]
async fn list_repos(&self) -> Result<CallToolResult, McpError>;

#[tool(description = "Get full content of a file by path")]
async fn get_file(
    &self,
    #[arg(description = "Absolute path to file")] path: String,
) -> Result<CallToolResult, McpError>;

#[tool(description = "Get lines surrounding a match for context")]
async fn get_context(
    &self,
    #[arg(description = "Absolute path to file")] path: String,
    #[arg(description = "Line number to center on")] line: u32,
    #[arg(description = "Lines of context before and after")] context: Option<u32>,
) -> Result<CallToolResult, McpError>;

#[tool(description = "Index a new repository")]
async fn index_repo(
    &self,
    #[arg(description = "Path to repository")] path: String,
) -> Result<CallToolResult, McpError>;
```

- [x] Implement each tool with proper error handling
- [x] Return structured JSON that LLMs can parse easily

### Part 7.3: MCP Response Format

- [x] Design responses optimized for LLM consumption:
  ```json
  {
    "results": [
      {
        "file": "/abs/path/to/file.rs",
        "repo": "my-project",
        "language": "rust",
        "snippet": "fn process_data(...",
        "line": 42,
        "score": 0.95
      }
    ],
    "total_matches": 156,
    "query": "process data",
    "truncated": true,
    "hint": "Use get_file to see full content of relevant files"
  }
  ```
- [x] Add `truncated` flag when results exceed reasonable size
- [x] Include hints for follow-up actions
- [x] Limit response size to avoid overwhelming context windows

### Part 7.4: Integration Documentation

- [x] Create `doc/mcp-integration.md` with setup guides:
  - GitHub Copilot CLI: how to configure as tool
  - Claude Desktop: mcp.json configuration example
  - VS Code with Continue: extension settings
- [x] Shell aliases provided in --help output:
  ```bash
  alias ki='knowledge-index'
  alias kis='knowledge-index search'
  alias kii='knowledge-index index .'
  ```
- [ ] *(Optional)* Document expected environment variables and paths

---

## Phase 8: Vector Search (Optional Enhancement)

Add semantic search capabilities alongside FTS5.

**Goal:** `knowledge-index search --semantic "find authentication logic"` returns conceptually relevant results.

**Note:** This phase is optional and adds significant complexity. Evaluate need before implementing.

### Part 8.1: Architecture Decision

- [x] Evaluate trade-offs:
  | Approach | Pros | Cons |
  |----------|------|------|
  | sqlite-vec | Single DB file, no deps | Requires embedding model |
  | Local ONNX model | Offline, fast | ~100MB model size, CPU intensive |
  | API-based embeddings | Best quality | Requires network, costs money |
- [x] Recommended: Start with `sqlite-vec` + local ONNX model
- [x] Make vector search opt-in via `config.enable_semantic_search`

### Part 8.2: Embedding Pipeline

- [x] Add `fastembed` crate for embedding generation (uses ONNX internally)
- [x] Choose model: `all-MiniLM-L6-v2` (22MB, good quality/speed)
- [x] Download model on first use to config directory
- [x] Create embedding function:
  ```rust
  fn embed_text(model: &Session, text: &str) -> Result<Vec<f32>> {
      // Tokenize, run inference, return 384-dim vector
  }
  ```
- [x] Chunk large files into ~512 token segments with overlap
- [x] Store embeddings during indexing (when enabled)

### Part 8.3: Vector Storage

- [x] Store embeddings as binary blobs in SQLite (no extension needed)
- [x] Extend schema:
  ```sql
  CREATE TABLE IF NOT EXISTS embeddings (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      file_id INTEGER NOT NULL,
      chunk_index INTEGER NOT NULL,
      start_offset INTEGER NOT NULL,
      end_offset INTEGER NOT NULL,
      chunk_text TEXT NOT NULL,
      embedding BLOB NOT NULL
  );
  ```
- [x] Cosine similarity calculation in Rust
- [x] Handle schema migration when feature is enabled

### Part 8.4: Hybrid Search

- [x] Implement similarity search:
  ```rust
  fn vector_search(&self, query_embedding: &[f32], ...) -> Result<Vec<VectorSearchResult>>
  ```
- [x] Implement Reciprocal Rank Fusion:
  ```rust
  fn fuse_results(fts_results: Vec<(Id, f64)>, vec_results: Vec<(Id, f64)>, k: f64) -> Vec<(Id, f64)> {
      // RRF formula: score = sum(1 / (k + rank))
      // Combine rankings from both sources
  }
  ```
- [x] Add flags:
  - `--semantic` — vector search only
  - `--hybrid` — combine FTS and vector (default when enabled)
  - `--lexical` — FTS only (always available)

### Part 8.5: Performance Considerations

- [x] Embedding generation is slow; show progress
- [ ] *(Optional)* Consider async embedding to not block UI
- [x] Cache embeddings aggressively (hash-based invalidation)
- [x] Provide `knowledge-index rebuild-embeddings` for regeneration

---

## Phase 9: Remote Capabilities (Future Outlook)

Prepare architecture for remote usage without implementing initially.

**Goal:** Document design decisions that enable future remote features without blocking current development.

**Note:** This phase contains architectural guidelines and future designs. Items marked as *(Future)* are not required for initial release.

### Part 9.1: Architecture Preparation

Current implementation should follow these patterns to enable future remote support:

- [ ] *(Future)* Abstract database access behind trait:
  ```rust
  #[async_trait]
  pub trait IndexStore {
      async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>>;
      async fn list_repos(&self) -> Result<Vec<Repository>>;
      async fn add_repo(&self, path: &Path) -> Result<Repository>;
      // ...
  }
  
  // Current implementation
  pub struct SqliteStore { /* ... */ }
  
  // Future implementation
  pub struct RemoteStore { client: HttpClient, base_url: Url }
  ```
- [ ] *(Future)* Keep core logic independent of storage implementation
- [x] Use async where practical (tokio) to enable network I/O later (MCP server uses tokio)
- [x] Document current decisions that affect remote: 
  - Single-user assumption
  - Local file paths in results
  - No authentication

### Part 9.2: Future Server Mode Design

**Not implemented in initial release**, but documented for planning:

- [ ] *(Future)* HTTP API surface (future):
  ```
  POST /api/search       - Search indexed content
  GET  /api/repos        - List repositories
  POST /api/repos        - Add repository
  DELETE /api/repos/:id  - Remove repository
  GET  /api/files/:path  - Get file content
  ```
- [ ] *(Future)* SSE transport for MCP (remote AI tools)
- [ ] *(Future)* Authentication options to consider:
  - API keys (simple)
  - OAuth (enterprise)
  - mTLS (zero-trust)
- [ ] *(Future)* Multi-user considerations:
  - Per-user indexes vs shared
  - Permission model for repositories
- [ ] *(Future)* SQLite sync options:
  - Litestream for backups
  - libSQL for replication
  - Full database sync on connect

### Part 9.3: Configuration Flags (Reserved)

Reserve these config options for future:
```toml
[remote]
enabled = false
# server_url = "https://..."
# api_key = "..."
# sync_mode = "manual"  # or "auto"
```

---

## Phase 10: Polish and Release

Final quality improvements and release preparation.

**Goal:** Production-ready release with documentation and distribution.

**Note:** This phase contains polish items for a 1.0 release. Items marked as *(Pre-release)* should be completed before major releases.

### Part 10.1: Performance Optimization

- [ ] *(Pre-release)* Profile with `cargo flamegraph` to identify hot paths
- [ ] *(Optional)* Optimize indexing:
  - Tune batch size for transaction commits
  - Consider memory-mapped file reading for large files
  - Parallel directory traversal vs parallel file processing
- [ ] *(Optional)* Optimize search:
  - Ensure FTS5 indexes are optimized
  - Consider query caching for repeated searches
  - Measure and document typical query latencies
- [ ] *(Optional)* Add memory limits:
  - Cap file content buffer size
  - Limit results held in memory
  - Use streaming for large result sets
- [ ] *(Pre-release)* Benchmark targets:
  - 100k files indexed in <30s on modern hardware
  - Search results in <100ms for typical queries
  - Memory usage <100MB during normal operation

### Part 10.2: Error Handling Polish

- [x] Audit all `unwrap()` and `expect()` calls (only 2 found, both safe: ProgressStyle templates)
- [x] Document invariants for remaining unwraps
- [x] Ensure errors include context (uses `anyhow` with context):
  ```rust
  .with_context(|| format!("Failed to read file: {}", path.display()))?
  ```
- [x] `--debug` flag for verbose error output with backtraces
- [ ] *(Pre-release)* Test error paths:
  - Permission denied on files/directories
  - Disk full during indexing
  - Corrupted database
  - Invalid config file
  - Network filesystem timeouts

### Part 10.3: Testing Strategy

- [x] Unit tests (18 tests covering core modules):
  - Config parsing and defaults
  - Query building and escaping
  - Search mode handling
  - Markdown parsing
  - Platform utilities
- [x] Integration tests:
  - CLI help, version, config, list, search commands
  - Full index → search cycle (optional test)
  - Uses tempfile for isolated testing
- [ ] *(Optional)* TUI tests:
  - Consider `insta` for snapshot testing
  - Or manual test script with expected behaviors
- [x] CI pipeline (`.github/workflows/ci.yml`):
  - Multi-platform: Linux, macOS, Windows
  - Rust stable and beta channels
  - MSRV check (1.75)
  - Format check, clippy, tests, doc tests
  - Publish dry-run

### Part 10.4: Documentation

- [x] Update README.md:
  - Add motivation section
  - Installation methods (cargo, homebrew, etc.)
  - Quick start examples
  - Feature overview with screenshots/gifs
  - Link to full documentation
- [ ] *(Optional)* Generate man page from clap:
  ```rust
  // build.rs
  use clap_mangen::Man;
  ```
- [x] Create `doc/` content:
  - `documentation.md` — detailed usage guide
  - `mcp-integration.md` — AI tool setup
  - `configuration.md` — all config options (in documentation.md)
  - `troubleshooting.md` — common issues (in documentation.md)
- [x] Add inline documentation (`///` doc comments) for public API
- [ ] *(Optional)* Record terminal demo with `asciinema` or similar

### Part 10.5: Distribution

- [x] Prepare for crates.io:
  - Complete `Cargo.toml` metadata (keywords, categories, repository, MSRV)
  - All dependencies are published
  - `cargo publish --dry-run` verified
- [x] GitHub releases workflow:
  - Cross-platform binary builds
  - Build binaries for: 
    - x86_64-unknown-linux-gnu
    - x86_64-apple-darwin
    - aarch64-apple-darwin
    - x86_64-pc-windows-msvc
  - Checksum generation
  - Automatic GitHub releases on tags
  - crates.io publish on stable tags
- [x] MIT LICENSE file added
- [ ] *(Optional)* Package managers:
  - Homebrew formula (create tap or submit to homebrew-core)
  - AUR PKGBUILD
  - Scoop manifest
  - Nix flake (optional)
- [x] Create CHANGELOG.md following Keep a Changelog format (progress.md serves this purpose)

---

## Brainstormed Additional Features

Ideas for future consideration:

### Search Enhancements
- **Fuzzy matching** — typo tolerance for search queries
- **Regex search** — advanced pattern matching mode
- **Code symbol extraction** — index function/class names separately for faster lookup
- **Search history** — remember recent queries in TUI

### Knowledge Management Features
- **Backlink discovery** — find all notes that link to a given note (`[[note]]`)
- **Tag browser** — browse and filter by frontmatter tags
- **Graph view** — visualize connections between notes (ASCII or external tool export)
- **Daily notes support** — recognize date-based files, enable date range searches
- **Orphan detection** — find notes with no incoming links
- **Broken link detection** — find `[[links]]` that point to non-existent files

### Repository Features
- **Git integration** — only index tracked files, show branch info
- **Repository groups** — organize repos into named collections (e.g., "work", "personal", "notes")
- **Priority indexing** — mark frequently-used repos for faster updates
- **Vault presets** — auto-detect Obsidian/Logseq/Dendron and apply appropriate settings

### AI Features
- **Context builder** — automatically gather relevant files for a prompt
- **Prompt templates** — pre-built prompts using search results
- **Summarization hints** — metadata to help AI summarize large results
- **Note-to-context** — given a note, find all related code and notes for AI context

### Developer Experience
- **Shell completions** — bash/zsh/fish autocompletion
- **Editor plugins** — VSCode, Neovim, Obsidian plugin for quick search
- **Watch mode** — `knowledge-index watch` for continuous terminal output
- **Alfred/Raycast integration** — quick launcher support on macOS

### Analytics
- **Usage stats** — most searched terms, frequently accessed files
- **Index health** — detect stale repos, orphaned entries
- **Knowledge stats** — total notes, tags, links, words indexed

---

## Recommended Crates

| Purpose | Crate | Version | Notes |
|---------|-------|---------|-------|
| CLI parsing | `clap` | 4.x | Use derive feature |
| TUI framework | `ratatui` | 0.28+ | Active fork of tui-rs |
| Terminal backend | `crossterm` | 0.28+ | Cross-platform |
| Database | `rusqlite` | 0.32+ | Use `bundled`, `modern_sqlite` features for FTS5 |
| Parallel processing | `rayon` | 1.x | Data parallelism |
| File walking | `ignore` | 0.4+ | Respects .gitignore, parallel |
| Hashing | `blake3` | 1.x | Fast content hashing |
| File watching | `notify` | 7.x | Use `macos_fsevent` feature |
| MCP protocol | `rmcp` | 0.8+ | Official Rust SDK |
| Config parsing | `toml` | 0.8+ | For config files |
| Serialization | `serde` + `serde_json` | 1.x | JSON/TOML support |
| Async runtime | `tokio` | 1.x | For MCP server, use `rt-multi-thread` |
| Logging | `tracing` + `tracing-subscriber` | 0.1 | Structured logging |
| Terminal colors | `owo-colors` | 4.x | Simple, supports `NO_COLOR` |
| Error handling | `thiserror` + `anyhow` | 1.x / 1.x | Custom errors + context |
| Progress bars | `indicatif` | 0.17+ | CLI progress display |
| Directories | `dirs` | 5.x | Cross-platform config paths |
| Time | `chrono` | 0.4+ | ISO 8601 timestamps |

### Optional (Phase 8)
| Purpose | Crate | Notes |
|---------|-------|-------|
| Vector storage | `sqlite-vec` | SQLite extension |
| ML inference | `ort` | ONNX Runtime bindings |

---

## Success Metrics

### Performance
- Index 100k files in under 30 seconds
- Search results in <100ms for typical queries
- Memory usage under 100MB during normal operation
- TUI renders at 60fps with no perceptible lag

### User Experience
- First successful search within 5 minutes of install
- Zero configuration needed for basic usage
- All common tasks achievable without documentation
- Error messages always actionable

### Quality
- Works offline, zero network dependencies for core features
- Single binary, no runtime dependencies
- Cross-platform: Linux, macOS, Windows
- No data loss or corruption under any circumstance

---

## Accessibility Considerations

### Visual
- [x] Respect `NO_COLOR` environment variable
- [x] Provide `--no-color` flag for all commands
- [x] Use semantic colors (not just red/green for status)
- [x] Ensure sufficient contrast in default theme
- [x] Support high-contrast terminal themes

### Motor
- [x] All actions accessible via keyboard
- [x] No time-sensitive interactions
- [x] Avoid requiring precise mouse movements
- [x] Support standard terminal shortcuts (Ctrl+C, etc.)

### Cognitive
- [x] Consistent navigation patterns across views
- [x] Clear, jargon-free error messages
- [x] Progressive disclosure (simple by default, advanced optional)
- [x] Confirmation for destructive actions (--force flag)

---

## Edge Cases & Error Scenarios

Document how to handle each scenario:

### File System
| Scenario | Behavior |
|----------|----------|
| File deleted during indexing | Skip, log warning, continue |
| Permission denied on file | Skip, include in "skipped" count |
| Symlink loop | Detect and skip, warn user |
| File modified during read | Use content as-read, hash will differ on next update |
| Disk full during indexing | Rollback transaction, show clear error |
| Network filesystem timeout | Retry once, then skip with warning |

### Database
| Scenario | Behavior |
|----------|----------|
| Corrupted database | Detect on open, offer to recreate |
| Database locked | Retry with backoff, timeout after 30s |
| Schema version mismatch | Run migrations automatically |
| Disk full | Fail gracefully, suggest cleanup |

### User Input
| Scenario | Behavior |
|----------|----------|
| Invalid path provided | Clear error with valid path example |
| Repository already indexed | Ask: update, skip, or force re-index |
| Empty search query | Show recent searches or hint |
| Regex syntax error | Show error with position indicator |

### System
| Scenario | Behavior |
|----------|----------|
| Terminal too small | Show resize message (Part 4.2) |
| No TTY (piped input) | CLI mode only, no TUI |
| Ctrl+C during operation | Clean shutdown, no corruption |
| Panic/crash | Restore terminal state, log stack trace |
