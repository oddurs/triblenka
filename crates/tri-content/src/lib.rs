//! Typed content: loaders, digests, and the store.
//!
//! Content is data, never code (`DESIGN.md`, Decision B). It is loaded into a digest-keyed store
//! and rendered by already-compiled templates, which is what keeps a content edit off `rustc` and
//! inside the millisecond budget.

mod loader;
mod post;
mod store;

pub use loader::{LoadReport, load_dir, load_file};
pub use post::Post;
pub use store::{Changed, Digest, Store};

/// Errors raised while loading or storing content.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The file could not be read or written.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    /// The store rejected the operation.
    #[error("store: {0}")]
    Store(String),
    /// A document's frontmatter is missing or malformed.
    #[error("{path}: {message}")]
    Frontmatter {
        /// File the problem is in.
        path: String,
        /// What is wrong, in terms a writer can act on.
        message: String,
    },
    /// A stored value could not be decoded.
    #[error("decode: {0}")]
    Decode(#[from] serde_json::Error),
}

/// Result alias used throughout the content layer.
pub type Result<T> = std::result::Result<T, Error>;
