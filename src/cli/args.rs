use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "knowledge-index",
    about = "Index and search code repositories and knowledge bases for AI-powered workflows",
    version,
    author
)]
#[command(after_help = "Examples:
  knowledge-index                     Launch interactive TUI
  knowledge-index index .             Index current directory
  knowledge-index index ~/notes       Index Obsidian vault
  knowledge-index search \"async fn\"   Search for async functions
  knowledge-index search \"TODO\" --type markdown
  knowledge-index list                List all indexed repositories

Shell Aliases (add to ~/.bashrc or ~/.zshrc):
  alias ki='knowledge-index'
  alias kis='knowledge-index search'
  alias kii='knowledge-index index .'
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
  knowledge-index index                    Index current directory
  knowledge-index index ~/projects/myapp   Index specific project
  knowledge-index index ~/Documents/notes  Index Obsidian vault
")]
    Index {
        /// Directory to index (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Custom name for the repository
        #[arg(long)]
        name: Option<String>,
    },

    /// Search indexed content
    #[command(after_help = "Examples:
  knowledge-index search \"database connection\"
  knowledge-index search \"async fn\" --repo api-service
  knowledge-index search \"TODO\" --type markdown
  knowledge-index search \"error handling\" --semantic
  knowledge-index search \"authentication\" --hybrid
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
  knowledge-index update .            Update current directory
  knowledge-index update --all        Update all repositories
")]
    Update {
        /// Repository path to update
        path: Option<PathBuf>,

        /// Update all indexed repositories
        #[arg(long)]
        all: bool,
    },

    /// List all indexed repositories
    List {},

    /// Remove a repository from the index
    #[command(after_help = "Examples:
  knowledge-index remove ~/projects/old-project
  knowledge-index remove . --force    Skip confirmation
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
        /// Configuration key to show/set
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
  knowledge-index rebuild-embeddings         Rebuild all embeddings
  knowledge-index rebuild-embeddings --repo myproject
")]
    RebuildEmbeddings {
        /// Filter by repository name
        #[arg(long)]
        repo: Option<String>,
    },
}
