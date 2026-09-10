//! The Triblenka rendering runtime.
//!
//! Rendering is a depth-first write into a [`Sink`]. There is no virtual DOM and no intermediate
//! tree: static markup is written straight through, and expressions are evaluated by compiled code
//! reached through the [`Bindings`] trait.
//!
//! A [`Template`] is the *descriptor* half of the design (`DESIGN.md` §5.1): a tree of static
//! chunks and indices into a table of compiled expressions. Release builds monomorphise it into
//! straight-line calls; the dev server walks it, which is what allows a template's markup to be
//! swapped at runtime without invoking `rustc`.

mod sink;
mod template;

pub use sink::{Html, Render, Sink, SinkExt, StringSink, escape_attr, escape_text};
pub use template::{Bindings, Node, Template, render};

/// Errors raised while rendering.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The descriptor referenced an expression the bindings do not provide.
    #[error("template referenced expression {index}, but the bindings provide {available}")]
    UnknownExpression {
        /// Index the descriptor asked for.
        index: usize,
        /// How many expressions the bindings actually provide.
        available: usize,
    },
    /// Writing into the sink failed.
    #[error("write failed: {0}")]
    Write(#[from] std::fmt::Error),
}

/// Result alias used throughout the runtime.
pub type Result<T> = std::result::Result<T, Error>;
