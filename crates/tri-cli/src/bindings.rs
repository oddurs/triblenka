//! Hand-written bindings, standing in for what codegen will emit.
//!
//! In a release build these are monomorphised straight-line calls generated from the `.tri` source
//! (`tri_compiler::codegen`). Until the `build.rs` loop exists, the benchmark dispatches on the
//! expression's source text instead. That is *slower* than the generated form, so the numbers it
//! produces are conservative — the real thing can only be faster.

use tri_content::Post;
use tri_core::{Bindings, Html, Result, Sink, SinkExt as _};

/// Bindings for one post page.
pub struct PostPage<'a> {
    /// The post being rendered.
    pub post: &'a Post,
    /// The descriptor's expression table.
    pub expressions: &'a [String],
}

impl Bindings for PostPage<'_> {
    fn value(&self, index: usize, sink: &mut dyn Sink) -> Result<()> {
        match self.expressions.get(index).map(String::as_str) {
            Some("post.title") => sink.escaped(&self.post.title),
            Some("post.date") => sink.escaped(&self.post.date),
            Some("post.description") => sink.escaped(&self.post.description),
            Some("post.slug") => sink.escaped(&self.post.slug),
            Some("post.body") => sink.escaped(&Html(&self.post.body_html)),
            _ => Ok(()),
        }
    }

    fn truthy(&self, index: usize) -> bool {
        matches!(
            self.expressions.get(index).map(String::as_str),
            Some("post.draft")
        ) && self.post.draft
    }
}

/// Bindings for the index page, which loops over posts.
pub struct IndexPage<'a> {
    /// Site title.
    pub title: &'a str,
    /// One binding per listed post.
    pub posts: Vec<PostPage<'a>>,
    /// The descriptor's expression table.
    pub expressions: &'a [String],
}

impl Bindings for IndexPage<'_> {
    fn value(&self, index: usize, sink: &mut dyn Sink) -> Result<()> {
        match self.expressions.get(index).map(String::as_str) {
            Some("site.title") => sink.escaped(&self.title),
            _ => Ok(()),
        }
    }

    fn seq_len(&self, index: usize) -> usize {
        if matches!(
            self.expressions.get(index).map(String::as_str),
            Some("posts")
        ) {
            self.posts.len()
        } else {
            0
        }
    }

    fn seq_item(&self, index: usize, item: usize) -> Option<&dyn Bindings> {
        if matches!(
            self.expressions.get(index).map(String::as_str),
            Some("posts")
        ) {
            self.posts.get(item).map(|p| p as &dyn Bindings)
        } else {
            None
        }
    }
}
