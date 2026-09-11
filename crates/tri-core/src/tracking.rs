//! Recording which expressions a render actually read.
//!
//! The build graph keys `rendered_page` on the *fields* a render observed rather than on the whole
//! entry, so editing a post's body does not rebuild an index that only ever read its title and
//! date.
//!
//! Recording happens in the walker, not in a [`Bindings`] wrapper. A wrapper cannot follow the
//! render into a loop body: [`Bindings::seq_item`] hands back a borrowed child, so there is nowhere
//! to put a wrapper around it, and reads inside the loop go unrecorded. That failure is silent and
//! it under-invalidates — exactly the shape of bug that serves a stale page. The walker sees every
//! index at every depth, so it is the only correct place for this.

use crate::{Bindings, Result, Sink, Template, template::walk_tracked};
use std::cell::RefCell;

/// The set of expression indices a render read, ascending.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Observed(Vec<usize>);

impl Observed {
    /// The indices read, ascending.
    #[must_use]
    pub fn indices(&self) -> &[usize] {
        &self.0
    }

    /// Whether any of `changed` was read. Both sides must be sorted; each is walked once.
    #[must_use]
    pub fn intersects(&self, changed: &[usize]) -> bool {
        let (mut a, mut b) = (0, 0);
        while a < self.0.len() && b < changed.len() {
            match self.0[a].cmp(&changed[b]) {
                std::cmp::Ordering::Equal => return true,
                std::cmp::Ordering::Less => a += 1,
                std::cmp::Ordering::Greater => b += 1,
            }
        }
        false
    }
}

/// Accumulates indices during a tracked render.
#[derive(Debug)]
pub(crate) struct Recorder {
    seen: RefCell<Vec<bool>>,
}

impl Recorder {
    pub(crate) fn new(expression_count: usize) -> Self {
        Self {
            seen: RefCell::new(vec![false; expression_count]),
        }
    }

    pub(crate) fn mark(&self, index: usize) {
        if let Some(slot) = self.seen.borrow_mut().get_mut(index) {
            *slot = true;
        }
    }

    pub(crate) fn finish(self) -> Observed {
        Observed(
            self.seen
                .into_inner()
                .iter()
                .enumerate()
                .filter_map(|(index, hit)| hit.then_some(index))
                .collect(),
        )
    }
}

/// Render, and report which expressions were read — at every depth, loop bodies included.
///
/// # Errors
///
/// Same failures as [`crate::render`].
pub fn render_tracked(
    template: &Template,
    bindings: &dyn Bindings,
    sink: &mut dyn Sink,
) -> Result<Observed> {
    let recorder = Recorder::new(template.expressions.len());
    walk_tracked(
        &template.nodes,
        template.expressions.len(),
        bindings,
        sink,
        &recorder,
    )?;
    Ok(recorder.finish())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::{Node, StringSink};

    struct Item;
    impl Bindings for Item {
        fn value(&self, _index: usize, sink: &mut dyn Sink) -> Result<()> {
            sink.raw("item")
        }
    }

    struct Page {
        items: Vec<Item>,
    }

    impl Bindings for Page {
        fn value(&self, _index: usize, sink: &mut dyn Sink) -> Result<()> {
            sink.raw("page")
        }
        fn truthy(&self, _index: usize) -> bool {
            false
        }
        fn seq_len(&self, index: usize) -> usize {
            if index == 1 { self.items.len() } else { 0 }
        }
        fn seq_item(&self, index: usize, item: usize) -> Option<&dyn Bindings> {
            if index == 1 {
                self.items.get(item).map(|i| i as &dyn Bindings)
            } else {
                None
            }
        }
    }

    #[test]
    fn records_reads_inside_a_loop_body() {
        // The regression this module exists for: a bindings wrapper cannot see index 2, because
        // the walker descends into a borrowed child it has no way to wrap.
        let template = Template {
            nodes: vec![
                Node::Expr(0),
                Node::For {
                    seq: 1,
                    body: vec![Node::Expr(2)],
                },
            ],
            expressions: vec!["site.title".into(), "posts".into(), "post.title".into()],
        };
        let page = Page {
            items: vec![Item, Item],
        };
        let mut sink = StringSink::new();
        let observed = render_tracked(&template, &page, &mut sink).expect("renders");

        assert_eq!(
            observed.indices(),
            &[0, 1, 2],
            "loop-body reads must be recorded"
        );
    }

    #[test]
    fn records_only_the_branch_that_ran() {
        let template = Template {
            nodes: vec![Node::If {
                cond: 0,
                then: vec![Node::Expr(1)],
                otherwise: vec![Node::Expr(2)],
            }],
            expressions: vec!["live".into(), "a".into(), "b".into()],
        };
        let page = Page { items: vec![] };
        let mut sink = StringSink::new();
        let observed = render_tracked(&template, &page, &mut sink).expect("renders");

        // `truthy` is false, so the else-branch ran: index 1 must not appear.
        assert_eq!(observed.indices(), &[0, 2]);
    }

    #[test]
    fn intersects_finds_a_shared_index() {
        let observed = Observed(vec![0, 2, 5]);
        assert!(observed.intersects(&[5]));
        assert!(observed.intersects(&[1, 2]));
        assert!(!observed.intersects(&[1, 3, 4]));
        assert!(!observed.intersects(&[]));
    }
}
