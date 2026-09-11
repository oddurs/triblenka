//! Experiment 0103: does field-level incrementality pay for itself?
//!
//! The claim under test is that keying a rendered page on the *fields* a render observed, rather
//! than on the whole entry, avoids rebuilding pages that did not read what changed. The canonical
//! case is an index: it reads each post's slug, title and date, so editing a body should not
//! rebuild it.
//!
//! What that has to beat is its own bookkeeping — a tracking wrapper on every render, a field diff
//! on every change, and a stored field set per page.

use crate::bindings::{IndexPage, PostPage};
use crate::fixture;
use std::time::{Duration, Instant};
use tri_content::{Post, Store};
use tri_core::{StringSink, Template, render_tracked};

const ITERATIONS: usize = 100;

/// Which expressions of `template` read a field that differs between `old` and `new`.
///
/// Returned sorted, because [`tri_core::Observed::intersects`] walks both sides once.
fn changed_expressions(template: &Template, old: &Post, new: &Post) -> Vec<usize> {
    let mut changed: Vec<&str> = Vec::new();
    if old.title != new.title {
        changed.push("post.title");
    }
    if old.date != new.date {
        changed.push("post.date");
    }
    if old.description != new.description {
        changed.push("post.description");
    }
    if old.slug != new.slug {
        changed.push("post.slug");
    }
    if old.draft != new.draft {
        changed.push("post.draft");
    }
    if old.body_html != new.body_html {
        changed.push("post.body");
    }

    let mut indices: Vec<usize> = template
        .expressions
        .iter()
        .enumerate()
        .filter_map(|(index, source)| changed.contains(&source.as_str()).then_some(index))
        .collect();
    indices.sort_unstable();
    indices
}

fn median(mut samples: Vec<Duration>) -> Duration {
    samples.sort_unstable();
    samples.get(samples.len() / 2).copied().unwrap_or_default()
}

/// Run the experiment against a fixture of `count` posts.
///
/// # Errors
///
/// Fails if the fixture cannot be written or the store cannot be opened.
pub fn run(count: usize) -> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::temp_dir().join("tri-experiment-0103");
    let content = root.join("content");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root)?;

    println!("Experiment 0103 — field-level incrementality, {count} posts\n");

    fixture::generate(&content, count)?;
    let store = Store::open(&root.join("store.redb"))?;
    tri_content::load_dir(&content, &store)?;

    let page = lower(fixture::PAGE_TEMPLATE);
    let index = lower(fixture::INDEX_TEMPLATE);
    let keys = store.keys()?;
    let posts: Vec<Post> = keys
        .iter()
        .filter_map(|k| store.get::<Post>(k).ok().flatten())
        .collect();
    let first = posts.first().ok_or("fixture produced no posts")?;

    // 1. What a field-level miss saves: not re-rendering the index.
    let index_cost = median(
        (0..ITERATIONS.min(20))
            .map(|_| {
                let start = Instant::now();
                let _ = render_index(&index, &posts);
                start.elapsed()
            })
            .collect(),
    );

    // 2. What it costs: the tracking wrapper on an ordinary page render.
    let untracked = median(
        (0..ITERATIONS)
            .map(|_| {
                let start = Instant::now();
                let bindings = PostPage {
                    post: first,
                    expressions: &page.expressions,
                };
                let mut sink = StringSink::with_capacity(8 * 1024);
                let _ = tri_core::render(&page, &bindings, &mut sink);
                start.elapsed()
            })
            .collect(),
    );

    let tracked = median(
        (0..ITERATIONS)
            .map(|_| {
                let start = Instant::now();
                let bindings = PostPage {
                    post: first,
                    expressions: &page.expressions,
                };
                let mut sink = StringSink::with_capacity(8 * 1024);
                let _ = render_tracked(&page, &bindings, &mut sink);
                start.elapsed()
            })
            .collect(),
    );

    // Render the index once, tracked, so the decision loop measures the decision and not a
    // 10,000-item render.
    let index_bindings_once = index_bindings(&index, &posts);
    let mut sink = StringSink::with_capacity(64 * 1024);
    let observed_index = render_tracked(&index, &index_bindings_once, &mut sink)?;

    // 3. The decision itself: diff the fields, then test the observed set.
    let mut edited = first.clone();
    edited.body_html.push_str("<p>edited</p>");
    let decision = median(
        (0..ITERATIONS)
            .map(|_| {
                let start = Instant::now();
                let changed = changed_expressions(&index, first, &edited);
                let _ = observed_index.intersects(&changed);
                start.elapsed()
            })
            .collect(),
    );

    // Correctness: a body-only edit must not touch what the index reads.
    let observed = observed_index;
    let body_change = changed_expressions(&index, first, &edited);
    let index_needs_rebuild = observed.intersects(&body_change);

    let mut retitled = first.clone();
    retitled.title = format!("{} (edited)", first.title);
    let title_change = changed_expressions(&index, first, &retitled);
    let index_needs_rebuild_on_title = observed.intersects(&title_change);

    println!("  index render ({count} items)             {index_cost:>10.3?}   ← saved on a miss");
    println!("  page render, untracked                 {untracked:>10.3?}");
    println!("  page render, tracked                   {tracked:>10.3?}");
    println!(
        "  tracking overhead                      {:>10.1}%",
        overhead_percent(untracked, tracked)
    );
    println!("  diff + intersect decision              {decision:>10.3?}\n");
    println!("  body edit  → index rebuilds? {index_needs_rebuild}  (must be false)");
    println!("  title edit → index rebuilds? {index_needs_rebuild_on_title}  (must be true)");

    // A 10,000-post fixture is ~200 MB of files; leaving several behind fills a disk.
    drop(store);
    let _ = std::fs::remove_dir_all(&root);

    let correct = !index_needs_rebuild && index_needs_rebuild_on_title;
    let worthwhile = index_cost > tracked.saturating_sub(untracked) + decision;
    println!(
        "\n  verdict: {}",
        if correct && worthwhile {
            "field-level tracking pays for itself"
        } else if correct {
            "correct, but the bookkeeping costs more than it saves at this size"
        } else {
            "INCORRECT — the tracker disagrees with the field diff"
        }
    );
    Ok(())
}

fn overhead_percent(untracked: Duration, tracked: Duration) -> f64 {
    if untracked.is_zero() {
        return 0.0;
    }
    (tracked.as_secs_f64() / untracked.as_secs_f64() - 1.0) * 100.0
}

fn index_bindings<'a>(template: &'a Template, posts: &'a [Post]) -> IndexPage<'a> {
    IndexPage {
        title: "Triblenka fixture",
        posts: posts
            .iter()
            .map(|post| PostPage {
                post,
                expressions: &template.expressions,
            })
            .collect(),
        expressions: &template.expressions,
    }
}

fn render_index(template: &Template, posts: &[Post]) -> String {
    let bindings = index_bindings(template, posts);
    let mut sink = StringSink::with_capacity(64 * 1024);
    let _ = tri_core::render(template, &bindings, &mut sink);
    sink.into_string()
}

fn lower(source: &str) -> Template {
    let parsed = tri_compiler::parse(source).unwrap_or_default();
    tri_compiler::descriptor::lower(&parsed)
}
