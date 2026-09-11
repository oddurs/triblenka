//! Lower a parsed document into a [`tri_core::Template`].
//!
//! Expressions are interned in first-seen order, so two parses of the same component produce the
//! same expression table whenever the expressions themselves are unchanged. That equality is what
//! [`tri_core::Template::is_hot_swappable_with`] tests, and therefore what decides whether a
//! template edit needs `rustc` or only a descriptor swap.

use crate::ast::{Document, Node};
use tri_core::{Node as RtNode, Template};

/// Lower a document to a runtime descriptor.
#[must_use]
pub fn lower(document: &Document) -> Template {
    let mut expressions = Vec::new();
    let nodes = lower_nodes(&document.nodes, &mut expressions);
    Template { nodes, expressions }
}

fn lower_nodes(nodes: &[Node], expressions: &mut Vec<String>) -> Vec<RtNode> {
    let mut out = Vec::with_capacity(nodes.len());
    for node in nodes {
        match node {
            Node::Text { value, .. } => match out.last_mut() {
                // Adjacent literals collapse: the fast path is one write per run of static markup.
                Some(RtNode::Static(previous)) => previous.push_str(value),
                _ => out.push(RtNode::Static(value.clone())),
            },
            Node::Expr {
                source, attribute, ..
            } => {
                let index = intern(expressions, source);
                out.push(if *attribute {
                    RtNode::ExprAttr(index)
                } else {
                    RtNode::Expr(index)
                });
            }
            Node::If {
                cond,
                then,
                otherwise,
                ..
            } => out.push(RtNode::If {
                cond: intern(expressions, cond),
                then: lower_nodes(then, expressions),
                otherwise: lower_nodes(otherwise, expressions),
            }),
            Node::For { seq, body, .. } => out.push(RtNode::For {
                seq: intern(expressions, seq),
                body: lower_nodes(body, expressions),
            }),
        }
    }
    out
}

fn intern(expressions: &mut Vec<String>, source: &str) -> usize {
    if let Some(index) = expressions.iter().position(|e| e == source) {
        return index;
    }
    expressions.push(source.to_owned());
    expressions.len() - 1
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::parse;

    fn template(src: &str) -> Template {
        lower(&parse(src).expect("parses"))
    }

    #[test]
    fn adjacent_static_text_collapses_into_one_write() {
        let t = template("<p>a</p>{#if x}<i>b</i>{/if}");
        assert!(matches!(t.nodes.first(), Some(RtNode::Static(s)) if s == "<p>a</p>"));
        assert_eq!(t.nodes.len(), 2);
    }

    #[test]
    fn repeated_expressions_share_one_index() {
        let t = template("{ title }-{ title }-{ date }");
        assert_eq!(t.expressions, vec!["title".to_owned(), "date".to_owned()]);
        assert_eq!(
            t.nodes
                .iter()
                .filter(|n| matches!(n, RtNode::Expr(0)))
                .count(),
            2
        );
    }

    #[test]
    fn markup_only_edits_stay_hot_swappable() {
        let before = template("<h1>{ title }</h1>{#for p in posts}<li>{ p.title }</li>{/for}");
        let after = template(
            "<header class=\"big\"><h1>{ title }</h1></header>\
             {#for p in posts}<article><li>{ p.title }</li></article>{/for}",
        );
        assert!(
            before.is_hot_swappable_with(&after),
            "changing only markup must not require a rebuild"
        );
    }

    #[test]
    fn changing_an_expression_requires_a_rebuild() {
        let before = template("<h1>{ title }</h1>");
        let after = template("<h1>{ subtitle }</h1>");
        assert!(!before.is_hot_swappable_with(&after));
    }

    #[test]
    fn adding_an_expression_requires_a_rebuild() {
        let before = template("<h1>{ title }</h1>");
        let after = template("<h1>{ title }</h1><p>{ description }</p>");
        assert!(!before.is_hot_swappable_with(&after));
    }
}
