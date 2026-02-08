mod embedder;
mod indexer;
mod markdown;
mod platform;
mod searcher;
mod watcher;

#[allow(unused_imports)]
pub use embedder::{ChunkEmbedding, Embedder, TextChunk};
pub use indexer::Indexer;
pub use markdown::parse_markdown;
#[allow(unused_imports)]
pub use markdown::{strip_markdown_syntax, CodeBlock, Heading, MarkdownMeta};
pub use platform::{check_inotify_limit, estimate_directory_count};
#[allow(unused_imports)]
pub use platform::PlatformLimits;
pub use searcher::{SearchMode, Searcher};
#[allow(unused_imports)]
pub use watcher::{ChangeType, IndexWatcher, PendingChange, RepoBatch};
