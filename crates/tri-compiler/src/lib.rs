//! The `.tri` compiler: parse a component, lower it to a descriptor, or emit Rust.
//!
//! One parse feeds two backends, which is what keeps the two render modes from drifting
//! (`DESIGN.md` §5.1):
//!
//! * [`descriptor::lower`] produces a [`tri_core::Template`] the dev server can swap at runtime.
//! * [`codegen::emit`] produces Rust source for release builds.
//!
//! Expressions are never evaluated here. The parser records their source text and span; making
//! them run is `rustc`'s job in release, and the bindings' job in dev.

pub mod ast;
pub mod codegen;
pub mod descriptor;
pub mod parser;

pub use ast::{Document, Node, ParseError, ParseErrorKind, Span};
pub use parser::parse;
