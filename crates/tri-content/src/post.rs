use serde::{Deserialize, Serialize};

/// A blog post, as stored.
///
/// The M0 spike hard-codes one collection. The `Collection` derive that generates this — along with
/// its schema, digest and typed query surface — is M1 work.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Post {
    /// URL slug, derived from the file stem.
    pub slug: String,
    /// Title, from frontmatter.
    pub title: String,
    /// Publication date, `YYYY-MM-DD`.
    pub date: String,
    /// Short description, from frontmatter.
    pub description: String,
    /// Whether the post is a draft.
    #[serde(default)]
    pub draft: bool,
    /// Tags, from frontmatter.
    #[serde(default)]
    pub tags: Vec<String>,
    /// The body, rendered to HTML at load time.
    pub body_html: String,
}

/// Frontmatter as written by an author, before defaults are applied.
#[derive(Debug, Deserialize)]
pub(crate) struct Frontmatter {
    pub title: String,
    pub date: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub tags: Vec<String>,
}
