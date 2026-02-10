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
  kdex index .             Index current directory
  kdex index ~/notes       Index Obsidian vault
  kdex add --remote owner/repo   Add remote GitHub repo
  kdex search \"async fn\"   Search for async functions
  kdex search \"TODO\" --type markdown
  kdex list                List all indexed repositories

Shell Aliases (add to ~/.bashrc or ~/.zshrc):
  alias ki='kdex'
  alias kis='kdex search'
  alias kii='kdex index .'
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
  kdex search \"TODO\" --type markdown
  kdex search \"error handling\" --semantic
  kdex search \"authentication\" --hybrid
")]
    Search {
        /// Search query (supports phrases and wildcards)
        query: String,

        /// Filter by repository name
        #[arg(long)]
        repo: Option<String>,

        /// Filter by file type (code, markdown, config)
        #[arg(long, name = "type")]
        file_type: Option<String>,

        /// Maximum number of results
        #[arg(long, default_value = "20")]
        limit: usize,

        /// Group results by repository
        #[arg(long)]
        group_by_repo: bool,

        /// Use semantic (vector) search
        #[arg(long, conflicts_with_all = ["hybrid", "lexical"])]
        semantic: bool,

        /// Use hybrid search (combines lexical + semantic)
        #[arg(long, conflicts_with_all = ["semantic", "lexical"])]
        hybrid: bool,

        /// Use lexical (full-text) search (default)
        #[arg(long, conflicts_with_all = ["semantic", "hybrid"])]
        lexical: bool,
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
