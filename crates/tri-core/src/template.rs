use crate::{Error, Result, Sink};

/// One node of a template descriptor.
///
/// Expressions are held as indices into a table of compiled code, never as source. The descriptor
/// orders compiled work; it never interprets Rust.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    /// A run of static markup, written through without escaping.
    Static(String),
    /// Write expression `index`, escaped for text content.
    Expr(usize),
    /// Write expression `index`, escaped for an attribute value.
    ///
    /// A separate node rather than a flag because the walker must wrap the sink, and because a
    /// descriptor that loses this distinction is an injection vector.
    ExprAttr(usize),
    /// Branch on expression `cond`.
    If {
        /// Index of the condition expression.
        cond: usize,
        /// Rendered when the condition holds.
        then: Vec<Node>,
        /// Rendered when it does not.
        otherwise: Vec<Node>,
    },
    /// Repeat `body` for each item of expression `seq`.
    For {
        /// Index of the sequence expression.
        seq: usize,
        /// Rendered once per item, against the item's own bindings.
        body: Vec<Node>,
    },
}

/// A parsed template: the shape of a component's output, with its expressions by index.
///
/// `expressions` holds the *source text* of each expression purely as an identity check. Two
/// parses whose expression lists match can swap descriptors at runtime; a change to any expression
/// means the compiled thunks are stale and a rebuild is required.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Template {
    /// The node tree.
    pub nodes: Vec<Node>,
    /// Source text of each expression, indexed as the nodes reference them.
    pub expressions: Vec<String>,
}

impl Template {
    /// Whether `other` can replace this descriptor without recompiling.
    ///
    /// True when the expression table is identical, which is exactly the condition under which the
    /// compiled thunks remain valid. This is the mechanism behind millisecond template reloads.
    #[must_use]
    pub fn is_hot_swappable_with(&self, other: &Template) -> bool {
        self.expressions == other.expressions
    }
}

/// The compiled half of a template: everything the descriptor cannot do itself.
///
/// An implementation is what codegen emits. Rendering never evaluates an expression — it asks the
/// bindings, which are ordinary compiled Rust.
pub trait Bindings {
    /// Write expression `index` into the sink, escaped.
    fn value(&self, index: usize, sink: &mut dyn Sink) -> Result<()>;

    /// Evaluate expression `index` as a condition.
    fn truthy(&self, _index: usize) -> bool {
        false
    }

    /// How many items expression `index` yields.
    fn seq_len(&self, _index: usize) -> usize {
        0
    }

    /// Bindings for item `item` of sequence `index`.
    fn seq_item(&self, _index: usize, _item: usize) -> Option<&dyn Bindings> {
        None
    }
}

/// Render a descriptor against its bindings.
///
/// # Errors
///
/// Returns [`Error::UnknownExpression`] if the descriptor references an expression the bindings do
/// not provide, and propagates write failures from the sink.
pub fn render(template: &Template, bindings: &dyn Bindings, sink: &mut dyn Sink) -> Result<()> {
    walk(&template.nodes, template.expressions.len(), bindings, sink)
}

fn walk(
    nodes: &[Node],
    expression_count: usize,
    bindings: &dyn Bindings,
    sink: &mut dyn Sink,
) -> Result<()> {
    for node in nodes {
        match node {
            Node::Static(text) => sink.raw(text)?,
            Node::Expr(index) => {
                check(*index, expression_count)?;
                bindings.value(*index, sink)?;
            }
            Node::ExprAttr(index) => {
                check(*index, expression_count)?;
                let mut wrapped = crate::AttributeSink(sink);
                bindings.value(*index, &mut wrapped)?;
            }
            Node::If {
                cond,
                then,
                otherwise,
            } => {
                check(*cond, expression_count)?;
                let branch = if bindings.truthy(*cond) {
                    then
                } else {
                    otherwise
                };
                walk(branch, expression_count, bindings, sink)?;
            }
            Node::For { seq, body } => {
                check(*seq, expression_count)?;
                for item in 0..bindings.seq_len(*seq) {
                    if let Some(scope) = bindings.seq_item(*seq, item) {
                        walk(body, expression_count, scope, sink)?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn check(index: usize, available: usize) -> Result<()> {
    if index < available {
        Ok(())
    } else {
        Err(Error::UnknownExpression { index, available })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::StringSink;

    struct Post(&'static str);
    impl Bindings for Post {
        fn value(&self, _index: usize, sink: &mut dyn Sink) -> Result<()> {
            sink.raw(self.0)
        }
    }

    struct Page {
        heading: &'static str,
        published: bool,
        posts: Vec<Post>,
    }

    impl Bindings for Page {
        fn value(&self, index: usize, sink: &mut dyn Sink) -> Result<()> {
            match index {
                0 => sink.raw(self.heading),
                _ => Ok(()),
            }
        }
        fn truthy(&self, index: usize) -> bool {
            index == 1 && self.published
        }
        fn seq_len(&self, index: usize) -> usize {
            if index == 2 { self.posts.len() } else { 0 }
        }
        fn seq_item(&self, index: usize, item: usize) -> Option<&dyn Bindings> {
            if index == 2 {
                self.posts.get(item).map(|p| p as &dyn Bindings)
            } else {
                None
            }
        }
    }

    fn template() -> Template {
        Template {
            nodes: vec![
                Node::Static("<h1>".into()),
                Node::Expr(0),
                Node::Static("</h1>".into()),
                Node::If {
                    cond: 1,
                    then: vec![Node::Static("<p>live</p>".into())],
                    otherwise: vec![Node::Static("<p>draft</p>".into())],
                },
                Node::For {
                    seq: 2,
                    body: vec![
                        Node::Static("<li>".into()),
                        Node::Expr(0),
                        Node::Static("</li>".into()),
                    ],
                },
            ],
            expressions: vec!["heading".into(), "published".into(), "posts".into()],
        }
    }

    fn render_page(page: &Page) -> String {
        let mut sink = StringSink::new();
        render(&template(), page, &mut sink).unwrap();
        sink.into_string()
    }

    #[test]
    fn walks_text_expressions_conditionals_and_loops() {
        let page = Page {
            heading: "Blog",
            published: true,
            posts: vec![Post("first"), Post("second")],
        };
        assert_eq!(
            render_page(&page),
            "<h1>Blog</h1><p>live</p><li>first</li><li>second</li>"
        );
    }

    #[test]
    fn takes_the_else_branch() {
        let page = Page {
            heading: "Blog",
            published: false,
            posts: vec![],
        };
        assert_eq!(render_page(&page), "<h1>Blog</h1><p>draft</p>");
    }

    #[test]
    fn out_of_range_expression_is_an_error_not_a_panic() {
        let template = Template {
            nodes: vec![Node::Expr(7)],
            expressions: vec!["only_one".into()],
        };
        let page = Page {
            heading: "x",
            published: false,
            posts: vec![],
        };
        let mut sink = StringSink::new();
        let err = render(&template, &page, &mut sink).unwrap_err();
        assert!(matches!(
            err,
            Error::UnknownExpression {
                index: 7,
                available: 1
            }
        ));
    }

    #[test]
    fn identical_expression_tables_are_hot_swappable() {
        let mut changed_markup = template();
        changed_markup
            .nodes
            .insert(0, Node::Static("<header/>".into()));
        assert!(template().is_hot_swappable_with(&changed_markup));

        let mut changed_expression = template();
        changed_expression.expressions[0] = "title".into();
        assert!(!template().is_hot_swappable_with(&changed_expression));
    }
}
