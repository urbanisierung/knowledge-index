//! MCP server implementation using rmcp.

use rmcp::{
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, ServerHandler, ServiceExt,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::Config;
use crate::core::{Embedder, SearchMode, Searcher};
use crate::db::Database;

/// MCP server for knowledge-index.
#[derive(Clone)]
pub struct KnowledgeIndexMcp {
    db: Arc<Mutex<Database>>,
    config: Arc<Config>,
}

/// Search result for MCP response.
#[derive(Debug, Serialize, Deserialize)]
struct McpSearchResult {
    file: String,
    repo: String,
    snippet: String,
    score: f64,
    mode: String,
}

/// Search response for MCP.
#[derive(Debug, Serialize, Deserialize)]
struct McpSearchResponse {
    results: Vec<McpSearchResult>,
    total: usize,
    query: String,
    mode: String,
    truncated: bool,
    hint: Option<String>,
}

/// Repository info for MCP.
#[derive(Debug, Serialize, Deserialize)]
struct McpRepoInfo {
    name: String,
    path: String,
    file_count: i64,
    status: String,
    last_indexed: Option<String>,
}

/// List repos response.
#[derive(Debug, Serialize, Deserialize)]
struct McpListReposResponse {
    repositories: Vec<McpRepoInfo>,
    total: usize,
}

/// Search request parameters.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchRequest {
    #[schemars(description = "Search query (keywords, phrases, or patterns)")]
    pub query: String,
    #[schemars(description = "Maximum number of results to return (default: 10, max: 50)")]
    pub limit: Option<u32>,
    #[schemars(description = "Filter by repository name")]
    pub repo: Option<String>,
    #[schemars(description = "Filter by file type (e.g., 'rust', 'markdown', 'python')")]
    pub file_type: Option<String>,
    #[schemars(description = "Search mode: 'lexical' (default), 'semantic', or 'hybrid'")]
    pub mode: Option<String>,
}

/// Get file request parameters.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetFileRequest {
    #[schemars(description = "Absolute path to the file")]
    pub path: String,
    #[schemars(description = "Maximum characters to return (default: 50000)")]
    pub max_chars: Option<u32>,
}

/// Get context request parameters.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetContextRequest {
    #[schemars(description = "Absolute path to the file")]
    pub path: String,
    #[schemars(description = "Line number to center the context around")]
    pub line: u32,
    #[schemars(description = "Number of context lines before and after (default: 10)")]
    pub context_lines: Option<u32>,
}

#[tool(tool_box)]
impl KnowledgeIndexMcp {
    /// Search indexed content across all repositories.
    #[tool(
        description = "Search indexed code and knowledge repositories for relevant content. Supports lexical (default), semantic (vector), or hybrid search modes."
    )]
    async fn search(&self, #[tool(aggr)] req: SearchRequest) -> String {
        let limit = req.limit.unwrap_or(10).min(50) as usize;
        let db = self.db.lock().await;

        // Determine search mode
        let search_mode = req.mode.as_deref().map_or_else(
            || SearchMode::from_str(&self.config.default_search_mode),
            SearchMode::from_str,
        );

        // Create searcher with embedder if needed
        let searcher = if (search_mode == SearchMode::Semantic || search_mode == SearchMode::Hybrid)
            && self.config.enable_semantic_search
        {
            match Embedder::new(&self.config.embedding_model) {
                Ok(embedder) => Searcher::with_embedder(db.clone(), embedder),
                Err(_) => Searcher::new(db.clone()),
            }
        } else {
            Searcher::new(db.clone())
        };

        // Use lexical if semantic requested but not available
        let effective_mode = if (search_mode == SearchMode::Semantic
            || search_mode == SearchMode::Hybrid)
            && !searcher.has_semantic_search()
        {
            SearchMode::Lexical
        } else {
            search_mode
        };

        let results = match searcher.search_with_mode(
            &req.query,
            effective_mode,
            req.repo.as_deref(),
            req.file_type.as_deref(),
            limit,
            0,
        ) {
            Ok(r) => r,
            Err(e) => return format!("{{\"error\": \"{e}\"}}"),
        };

        let total = results.len();
        let truncated = total >= limit;

        let mcp_results: Vec<McpSearchResult> = results
            .into_iter()
            .map(|r| McpSearchResult {
                file: r.absolute_path.to_string_lossy().to_string(),
                repo: r.repo_name,
                snippet: r.snippet,
                score: r.score,
                mode: r.search_mode.as_str().to_string(),
            })
            .collect();

        let response = McpSearchResponse {
            results: mcp_results,
            total,
            query: req.query,
            mode: effective_mode.as_str().to_string(),
            truncated,
            hint: if truncated {
                Some("Use 'limit' parameter to get more results, or use 'get_file' to read full content".into())
            } else {
                None
            },
        };

        serde_json::to_string_pretty(&response)
            .unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"))
    }

    /// List all indexed repositories.
    #[tool(description = "List all indexed repositories with their status and file counts")]
    async fn list_repos(&self) -> String {
        let db = self.db.lock().await;

        let repos = match db.list_repositories() {
            Ok(r) => r,
            Err(e) => return format!("{{\"error\": \"{e}\"}}"),
        };

        let repo_infos: Vec<McpRepoInfo> = repos
            .into_iter()
            .map(|r| McpRepoInfo {
                name: r.name,
                path: r.path.to_string_lossy().to_string(),
                file_count: r.file_count,
                status: r.status.as_str().to_string(),
                last_indexed: r.last_indexed_at.map(|t| t.to_rfc3339()),
            })
            .collect();

        let total = repo_infos.len();
        let response = McpListReposResponse {
            repositories: repo_infos,
            total,
        };

        serde_json::to_string_pretty(&response)
            .unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"))
    }

    /// Get full content of a file.
    #[allow(clippy::unused_self, clippy::needless_pass_by_value)]
    #[tool(description = "Get the full content of a specific file from the index")]
    fn get_file(&self, #[tool(aggr)] req: GetFileRequest) -> String {
        let max_chars = req.max_chars.unwrap_or(50000) as usize;

        let file_content = match std::fs::read_to_string(&req.path) {
            Ok(c) => c,
            Err(e) => return format!("{{\"error\": \"Failed to read file: {e}\"}}"),
        };

        let truncated = file_content.len() > max_chars;
        let content_str = if truncated {
            file_content.chars().take(max_chars).collect()
        } else {
            file_content
        };

        let file_type = std::path::Path::new(&req.path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("unknown");

        format!(
            "File: {}\nType: {file_type}\nTruncated: {truncated}\n\n{content_str}",
            req.path
        )
    }

    /// Get context around a specific line in a file.
    #[allow(clippy::unused_self, clippy::needless_pass_by_value)]
    #[tool(description = "Get lines of context around a specific line number in a file")]
    fn get_context(&self, #[tool(aggr)] req: GetContextRequest) -> String {
        let ctx_lines = req.context_lines.unwrap_or(10) as usize;

        let file_content = match std::fs::read_to_string(&req.path) {
            Ok(c) => c,
            Err(e) => return format!("Error: Failed to read file: {e}"),
        };

        let lines: Vec<&str> = file_content.lines().collect();
        let line_idx = (req.line as usize).saturating_sub(1);
        let start = line_idx.saturating_sub(ctx_lines);
        let end = (line_idx + ctx_lines + 1).min(lines.len());

        let formatted_lines: Vec<String> = lines[start..end]
            .iter()
            .enumerate()
            .map(|(i, l)| format!("{:4} | {l}", start + i + 1))
            .collect();

        format!(
            "File: {}\nLines {}-{end} (centered on line {}):\n\n{}",
            req.path,
            start + 1,
            req.line,
            formatted_lines.join("\n")
        )
    }
}

#[tool(tool_box)]
impl ServerHandler for KnowledgeIndexMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Search and retrieve content from indexed code repositories and knowledge bases. \
                 Use 'search' to find relevant files, 'list_repos' to see indexed repositories, \
                 'get_file' to read full file content, and 'get_context' to get context around \
                 specific lines."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

impl KnowledgeIndexMcp {
    /// Create a new MCP server instance.
    pub fn new(db: Database, config: Config) -> Self {
        Self {
            db: Arc::new(Mutex::new(db)),
            config: Arc::new(config),
        }
    }
}

/// Run the MCP server over stdio.
pub async fn run_mcp_server(db: Database, config: Config) -> crate::error::Result<()> {
    let server = KnowledgeIndexMcp::new(db, config);

    // Log to stderr only (stdout is for MCP protocol)
    print_mcp_startup_info();

    let service = server
        .serve(rmcp::transport::io::stdio())
        .await
        .map_err(|e| crate::error::AppError::Other(format!("MCP server error: {e}")))?;

    service
        .waiting()
        .await
        .map_err(|e| crate::error::AppError::Other(format!("MCP server error: {e}")))?;

    Ok(())
}

/// Print startup information and integration guide to stderr.
fn print_mcp_startup_info() {
    eprintln!("\x1b[1;36m╭─────────────────────────────────────────────────────────────╮\x1b[0m");
    eprintln!("\x1b[1;36m│\x1b[0m  \x1b[1mknowledge-index MCP Server\x1b[0m                                \x1b[1;36m│\x1b[0m");
    eprintln!("\x1b[1;36m╰─────────────────────────────────────────────────────────────╯\x1b[0m");
    eprintln!();
    eprintln!("\x1b[1mAvailable Tools:\x1b[0m");
    eprintln!("  \x1b[32m•\x1b[0m search       - Search indexed content (lexical/semantic/hybrid)");
    eprintln!("  \x1b[32m•\x1b[0m list_repos   - List all indexed repositories");
    eprintln!("  \x1b[32m•\x1b[0m get_file     - Read full file content");
    eprintln!("  \x1b[32m•\x1b[0m get_context  - Get lines around a specific line number");
    eprintln!();
    eprintln!("\x1b[1mIntegration:\x1b[0m");
    eprintln!();
    eprintln!("  \x1b[33mGitHub Copilot CLI\x1b[0m (~/.config/github-copilot/mcp.json):");
    eprintln!("  \x1b[90m┌──────────────────────────────────────────────────────────┐\x1b[0m");
    eprintln!("  \x1b[90m│\x1b[0m {{                                                         \x1b[90m│\x1b[0m");
    eprintln!("  \x1b[90m│\x1b[0m   \"mcpServers\": {{                                         \x1b[90m│\x1b[0m");
    eprintln!("  \x1b[90m│\x1b[0m     \"knowledge-index\": {{                                  \x1b[90m│\x1b[0m");
    eprintln!("  \x1b[90m│\x1b[0m       \"command\": \"knowledge-index\",                       \x1b[90m│\x1b[0m");
    eprintln!("  \x1b[90m│\x1b[0m       \"args\": [\"mcp\"]                                     \x1b[90m│\x1b[0m");
    eprintln!("  \x1b[90m│\x1b[0m     }}                                                      \x1b[90m│\x1b[0m");
    eprintln!("  \x1b[90m│\x1b[0m   }}                                                        \x1b[90m│\x1b[0m");
    eprintln!("  \x1b[90m│\x1b[0m }}                                                          \x1b[90m│\x1b[0m");
    eprintln!("  \x1b[90m└──────────────────────────────────────────────────────────┘\x1b[0m");
    eprintln!();
    eprintln!("  \x1b[33mClaude Desktop\x1b[0m (~/.config/claude/claude_desktop_config.json):");
    eprintln!("  \x1b[90m  Same configuration as above\x1b[0m");
    eprintln!();
    eprintln!("\x1b[1mStatus:\x1b[0m \x1b[32mListening on stdio...\x1b[0m");
    eprintln!("\x1b[90mPress Ctrl+C to stop\x1b[0m");
    eprintln!();
}
