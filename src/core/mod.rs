mod embedder;
mod indexer;
mod markdown;
mod platform;
pub mod remote;
mod searcher;
mod watcher;

#[allow(unused_imports)]
pub use embedder::{ChunkEmbedding, Embedder, TextChunk};
pub use indexer::Indexer;
pub use markdown::parse_markdown;
#[allow(unused_imports)]
pub use markdown::{strip_markdown_syntax, CodeBlock, Heading, MarkdownMeta};
#[allow(unused_imports)]
pub use platform::PlatformLimits;
pub use platform::{check_inotify_limit, estimate_directory_count};
pub use searcher::{SearchMode, Searcher};
#[allow(unused_imports)]
pub use watcher::{ChangeType, IndexWatcher, PendingChange, RepoBatch};
