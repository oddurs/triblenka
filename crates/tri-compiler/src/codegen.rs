//! Emit Rust source for a parsed document.
//!
//! This is the release half of the two render modes: static markup becomes one `sink.raw` call per
//! run, and expressions are pasted through verbatim so `rustc` type-checks them in place. The
//! generated code is never shown to a user — diagnostics are mapped back to the `.tri` source.

use crate::ast::{Document, Node};
use std::fmt::Write as _;

/// Emit a `render` function for `document`.
///
/// `name` becomes the component's module name.
#[must_use]
pub fn emit(name: &str, document: &Document) -> String {
    let mut out = String::with_capacity(1024);
    let _ = writeln!(out, "// @generated from {name}.tri — do not edit");
    let _ = writeln!(out, "pub mod {name} {{");
    let _ = writeln!(out, "    use tri_core::{{Render, Result, Sink}};");
    out.push_str("\n    #[allow(unused_variables)]\n");
    let _ = writeln!(
        out,
        "    pub fn render(sink: &mut dyn Sink) -> Result<()> {{"
    );
    if let Some(frontmatter) = &document.frontmatter {
        for line in frontmatter.lines() {
            let _ = writeln!(out, "        {line}");
        }
        out.push('\n');
    }
    emit_nodes(&document.nodes, 2, &mut out);
    let _ = writeln!(out, "        Ok(())");
    let _ = writeln!(out, "    }}");
    let _ = writeln!(out, "}}");
    out
}

fn emit_nodes(nodes: &[Node], depth: usize, out: &mut String) {
    let pad = "    ".repeat(depth);
    let mut pending = String::new();

    for node in nodes {
        if let Node::Text { value, .. } = node {
            pending.push_str(value);
            continue;
        }
        flush(&mut pending, &pad, out);
        match node {
            Node::Text { .. } => unreachable!("handled above"),
            Node::Expr { source, .. } => {
                let _ = writeln!(out, "{pad}sink.escaped(&({source}))?;");
            }
            Node::If {
                cond,
                then,
                otherwise,
                ..
            } => {
                let _ = writeln!(out, "{pad}if {cond} {{");
                emit_nodes(then, depth + 1, out);
                if otherwise.is_empty() {
                    let _ = writeln!(out, "{pad}}}");
                } else {
                    let _ = writeln!(out, "{pad}}} else {{");
                    emit_nodes(otherwise, depth + 1, out);
                    let _ = writeln!(out, "{pad}}}");
                }
            }
            Node::For {
                binding, seq, body, ..
            } => {
                let _ = writeln!(out, "{pad}for {binding} in {seq} {{");
                emit_nodes(body, depth + 1, out);
                let _ = writeln!(out, "{pad}}}");
            }
        }
    }
    flush(&mut pending, &pad, out);
}

fn flush(pending: &mut String, pad: &str, out: &mut String) {
    if pending.is_empty() {
        return;
    }
    let _ = writeln!(out, "{pad}sink.raw({})?;", quote(pending));
    pending.clear();
}

fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn static_runs_collapse_to_one_write() {
        let doc = parse("<p>hello</p><p>world</p>").expect("parses");
        let code = emit("greeting", &doc);
        assert_eq!(code.matches("sink.raw(").count(), 1);
        assert!(
            code.contains(r#"sink.raw("<p>hello</p><p>world</p>")?;"#),
            "{code}"
        );
    }

    #[test]
    fn expressions_are_pasted_verbatim_for_rustc() {
        let doc = parse("<h1>{ post.title.to_uppercase() }</h1>").expect("parses");
        let code = emit("page", &doc);
        assert!(
            code.contains("sink.escaped(&(post.title.to_uppercase()))?;"),
            "{code}"
        );
    }

    #[test]
    fn control_flow_becomes_real_rust() {
        let doc =
            parse("{#if p.live}<b>{ p.title }</b>{:else}draft{/if}{#for p in posts}<li/>{/for}")
                .expect("parses");
        let code = emit("list", &doc);
        assert!(code.contains("if p.live {"), "{code}");
        assert!(code.contains("} else {"), "{code}");
        assert!(code.contains("for p in posts {"), "{code}");
    }

    #[test]
    fn frontmatter_is_lifted_into_the_function_body() {
        let doc = parse("---\nlet posts = content::blog();\n---\n<p/>").expect("parses");
        let code = emit("index", &doc);
        assert!(code.contains("let posts = content::blog();"), "{code}");
    }

    #[test]
    fn quotes_and_newlines_survive_the_string_literal() {
        let doc = parse("<a class=\"x\">\n\tlink\n</a>").expect("parses");
        let code = emit("link", &doc);
        assert!(code.contains(r#"class=\"x\""#), "{code}");
        assert!(code.contains("\\n"), "{code}");
    }
}
