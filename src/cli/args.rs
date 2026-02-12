use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "kdex",
    about = "Index and search code repositories and knowledge bases for AI-powered workflows",
    version,
    author
)]
#[command(after_help = "Examples:
  kdex                     Launch interactive TUI
  kdex \"async fn\"          Search for async functions (default)
  kdex TODO --type markdown Search TODOs in markdown files
  kdex index .             Index current directory
  kdex add --remote owner/repo   Add remote GitHub repo
  kdex list                List all indexed repositories

The default command is 'search' - just type your query directly:
  kdex \"my query\"    →    kdex search \"my query\"

Shell Aliases (add to ~/.bashrc or ~/.zshrc):
  alias k='kdex'
  alias kx='kdex'
")]
#[allow(clippy::struct_excessive_bools)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Output as JSON
    #[arg(long, global = true)]
    pub json: bool,

    /// Suppress non-error output
    #[arg(long, global = true)]
    pub quiet: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Enable verbose output
    #[arg(long, short, global = true)]
    pub verbose: bool,

    /// Enable debug output with backtraces
    #[arg(long, global = true)]
    pub debug: bool,
}

#[derive(Subcommand, Clone)]
pub enum Commands {
    /// Index a directory (code repository or knowledge base)
    #[command(after_help = "Examples:
  kdex index                    Index current directory
  kdex index ~/projects/myapp   Index specific project
  kdex index ~/Documents/notes  Index Obsidian vault
")]
    Index {
        /// Directory to index (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Custom name for the repository
        #[arg(long)]
        name: Option<String>,
    },

    /// Add a repository (local or remote GitHub)
    #[command(after_help = "Examples:
  kdex add .                      Add local directory
  kdex add --remote owner/repo    Add GitHub repo by shorthand
  kdex add --remote https://github.com/owner/repo
  kdex add --remote owner/repo --branch develop
  kdex add --remote owner/repo --shallow
")]
    Add {
        /// Local directory path (when not using --remote)
        path: Option<PathBuf>,

        /// Add a remote GitHub repository
        #[arg(long, short)]
        remote: Option<String>,

        /// Branch to clone (for remote repos)
        #[arg(long)]
        branch: Option<String>,

        /// Use shallow clone (faster, less disk space)
        #[arg(long)]
        shallow: bool,

        /// Custom name for the repository
        #[arg(long)]
        name: Option<String>,
    },

    /// Search indexed content
    #[command(after_help = "Examples:
  kdex search \"database connection\"
  kdex search \"async fn\" --repo api-service
  kdex search \"TODO\" --file-type markdown
  kdex search \"error handling\" --semantic
  kdex search \"authentication\" --hybrid

Or use the shorthand (search is the default command):
  kdex \"database connection\"
  kdex TODO -t markdown
")]
    Search {
        /// Search query (supports phrases and wildcards)
        query: String,

        /// Filter by repository name
        #[arg(long, short)]
        repo: Option<String>,

        /// Filter by file type (code, markdown, config)
        #[arg(long, short = 't')]
        file_type: Option<String>,

        /// Filter by tag (from frontmatter)
        #[arg(long)]
        tag: Option<String>,

        /// Maximum number of results
        #[arg(long, short, default_value = "20")]
        limit: usize,

        /// Group results by repository
        #[arg(long, short = 'g')]
        group_by_repo: bool,

        /// Use semantic (vector) search
        #[arg(long, short = 's', conflicts_with_all = ["hybrid", "lexical", "fuzzy", "regex"])]
        semantic: bool,

        /// Use hybrid search (combines lexical + semantic)
        #[arg(long, short = 'H', conflicts_with_all = ["semantic", "lexical", "fuzzy", "regex"])]
        hybrid: bool,

        /// Use lexical (full-text) search (default)
        #[arg(long, conflicts_with_all = ["semantic", "hybrid", "fuzzy", "regex"])]
        lexical: bool,

        /// Use fuzzy matching (tolerates typos)
        #[arg(long, conflicts_with_all = ["semantic", "hybrid", "lexical", "regex"])]
        fuzzy: bool,

        /// Use regex pattern matching
        #[arg(long, conflicts_with_all = ["semantic", "hybrid", "lexical", "fuzzy"])]
        regex: bool,
    },

    /// Update an existing index
    #[command(after_help = "Examples:
  kdex update .            Update current directory
  kdex update --all        Update all repositories
")]
    Update {
        /// Repository path to update
        path: Option<PathBuf>,

        /// Update all indexed repositories
        #[arg(long)]
        all: bool,
    },

    /// Sync remote repositories with their origins
    #[command(after_help = "Examples:
  kdex sync                Sync all remote repositories
  kdex sync owner/repo     Sync specific remote repository
")]
    Sync {
        /// Specific repository to sync (by name or path)
        repo: Option<String>,

        /// Skip re-indexing after sync
        #[arg(long)]
        no_index: bool,
    },

    /// List all indexed repositories
    List {},

    /// Remove a repository from the index
    #[command(after_help = "Examples:
  kdex remove ~/projects/old-project
  kdex remove . --force    Skip confirmation
")]
    Remove {
        /// Repository path to remove
        path: PathBuf,

        /// Skip confirmation prompt
        #[arg(long, short)]
        force: bool,
    },

    /// Show or edit configuration
    Config {
        #[command(subcommand)]
        action: Option<ConfigAction>,

        /// Configuration key to show/set (legacy, use subcommands instead)
        key: Option<String>,

        /// Value to set
        value: Option<String>,

        /// Reset to defaults
        #[arg(long)]
        reset: bool,
    },

    /// Start MCP server for AI tool integration
    Mcp {},

    /// Watch for file changes and re-index automatically
    Watch {
        /// Watch all indexed repositories
        #[arg(long)]
        all: bool,

        /// Specific repository path to watch
        path: Option<PathBuf>,
    },

    /// Rebuild embeddings for semantic search
    #[command(after_help = "Examples:
  kdex rebuild-embeddings         Rebuild all embeddings
  kdex rebuild-embeddings --repo myproject
")]
    RebuildEmbeddings {
        /// Filter by repository name
        #[arg(long)]
        repo: Option<String>,
    },

    /// Generate shell completions
    #[command(after_help = "Examples:
  kdex completions bash > ~/.local/share/bash-completion/completions/kdex
  kdex completions zsh > ~/.zfunc/_kdex
  kdex completions fish > ~/.config/fish/completions/kdex.fish
")]
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },

    /// Find files that link to a target file (backlinks)
    #[command(after_help = "Examples:
  kdex backlinks my-note.md      Find files linking to my-note
  kdex backlinks project-idea    Find backlinks by stem name
")]
    Backlinks {
        /// Target file to find backlinks for
        file: PathBuf,
    },

    /// List all tags from indexed files
    #[command(after_help = "Extracts tags from YAML frontmatter in markdown files.")]
    Tags,

    /// Build AI context from search results
    #[command(after_help = "Examples:
  kdex context \"authentication\"         Build context for AI prompt
  kdex context \"error handling\" -l 5    Limit to 5 files
  kdex context \"api design\" --tokens 2000  Limit by tokens
")]
    Context {
        /// Search query to find relevant files
        query: String,

        /// Maximum number of files to include
        #[arg(long, short, default_value = "10")]
        limit: usize,

        /// Maximum approximate tokens
        #[arg(long, default_value = "4000")]
        tokens: usize,

        /// Output format (markdown, text, json)
        #[arg(long, default_value = "markdown")]
        format: String,
    },

    /// Show knowledge index statistics
    Stats {},

    /// Export knowledge graph visualization
    #[command(after_help = "Examples:
  kdex graph                    Output DOT format (for Graphviz)
  kdex graph --json             Output JSON for web visualization
  kdex graph --repo myproject   Graph only one repository
  kdex graph > graph.dot && dot -Tpng graph.dot -o graph.png
")]
    Graph {
        /// Output format (dot, json)
        #[arg(long, default_value = "dot")]
        format: String,

        /// Filter by repository name
        #[arg(long, short)]
        repo: Option<String>,
    },

    /// Check knowledge index health
    #[command(after_help = "Examples:
  kdex health                   Run all health checks
  kdex health --repo myproject  Check specific repository
  kdex health --json            Output as JSON
")]
    Health {
        /// Filter by repository name
        #[arg(long, short)]
        repo: Option<String>,
    },

    /// Configure MCP integration for AI tools
    #[command(after_help = "Examples:
  kdex add-mcp copilot    Configure GitHub Copilot CLI
  kdex add-mcp gemini     Configure Gemini CLI
  kdex add-mcp claude     Configure Claude Desktop

Supported tools: copilot, gemini, claude
")]
    AddMcp {
        /// AI tool to configure (copilot, gemini, claude)
        tool: McpTool,
    },

    /// Update kdex to the latest version
    #[command(after_help = "Re-runs the install script to update kdex.
Only works if kdex was installed via the install script.

For other installation methods:
  cargo install kdex      # If installed via cargo
  Download from GitHub    # For manual binary installs
")]
    SelfUpdate,
}

/// AI tool for MCP configuration
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum McpTool {
    /// GitHub Copilot CLI
    Copilot,
    /// Gemini CLI
    Gemini,
    /// Claude Desktop
    Claude,
}

/// Shell type for completions
#[derive(Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
#[allow(clippy::enum_variant_names)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Elvish,
}

#[derive(Subcommand, Clone)]
pub enum ConfigAction {
    /// Show current configuration
    Show,

    /// Export configuration to file (for backup/migration)
    #[command(after_help = "Examples:
  kdex config export                     Export to stdout
  kdex config export -o kdex-config.yaml Export to file
  kdex config export --remotes-only      Only remote repos (portable)
")]
    Export {
        /// Output file (default: stdout)
        #[arg(long, short)]
        output: Option<PathBuf>,

        /// Only export remote repositories (portable)
        #[arg(long)]
        remotes_only: bool,

        /// Include local repositories (with path warning)
        #[arg(long)]
        include_local: bool,

        /// Output format
        #[arg(long, default_value = "yaml")]
        format: String,
    },

    /// Import configuration from file
    #[command(after_help = "Examples:
  kdex config import kdex-config.yaml
  kdex config import kdex-config.yaml --merge
  cat config.yaml | kdex config import -
")]
    Import {
        /// Input file (use '-' for stdin)
        file: PathBuf,

        /// Merge with existing config instead of replacing
        #[arg(long)]
        merge: bool,

        /// Skip cloning remote repositories
        #[arg(long)]
        skip_clone: bool,
    },
}
