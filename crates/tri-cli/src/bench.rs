//! The M0 measurement harness.
//!
//! Two kill criteria, from `DESIGN.md` §16:
//!
//! 1. a content-only rebuild lands under **50 ms**;
//! 2. a structure-only template edit lands under **100 ms** end to end.
//!
//! Either one failing stops the project, so the harness reports p50 and p99 rather than a mean —
//! a budget met on average is not met.

use crate::bindings::{IndexPage, PostPage};
use crate::fixture;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tri_content::{Post, Store};
use tri_core::{StringSink, Template};

/// Enough samples that the 99th percentile is a percentile rather than the maximum: with 50
/// samples the p99 index rounds to the last one, which overstates what was measured.
const ITERATIONS: usize = 200;

/// The rescan is two orders of magnitude slower, so it gets fewer samples.
const RESCAN_ITERATIONS: usize = 20;

/// Outcome of one measured loop.
struct Measurements {
    samples: Vec<Duration>,
}

impl Measurements {
    fn percentile(&self, p: f64) -> Duration {
        let mut sorted = self.samples.clone();
        sorted.sort_unstable();
        let index = ((sorted.len() as f64 - 1.0) * p).round() as usize;
        sorted.get(index).copied().unwrap_or_default()
    }

    fn report(&self, name: &str, budget: Duration) -> bool {
        let p50 = self.percentile(0.50);
        let p99 = self.percentile(0.99);
        let pass = p99 <= budget;
        println!(
            "  {name:<38} p50 {:>8.3?}  p99 {:>8.3?}  budget {:>6.0?}  {}",
            p50,
            p99,
            budget,
            if pass { "PASS" } else { "FAIL" }
        );
        pass
    }
}

/// Run both kill-criterion measurements against a fixture of `count` posts.
///
/// # Errors
///
/// Fails if the fixture cannot be written or the store cannot be opened.
pub fn run(count: usize) -> Result<bool, Box<dyn std::error::Error>> {
    let root = std::env::temp_dir().join("tri-m0-bench");
    let content = root.join("content");
    let out = root.join("dist");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&out)?;

    println!(
        "M0 kill criteria — {count} posts, {ITERATIONS} iterations ({RESCAN_ITERATIONS} for the rescan)\n"
    );

    fixture::generate(&content, count)?;
    let store = Store::open(&root.join("store.redb"))?;

    let template = lower(fixture::PAGE_TEMPLATE);
    let index_template = lower(fixture::INDEX_TEMPLATE);

    // Cold build, so the measured loops are all warm-cache incremental work.
    let cold = Instant::now();
    let report = tri_content::load_dir(&content, &store)?;
    let keys = store.keys()?;
    for key in &keys {
        if let Some(post) = store.get::<Post>(key)? {
            write_page(&out, &post, &template)?;
        }
    }
    write_index(&out, &store, &keys, &index_template)?;
    println!(
        "  cold build                             {:>8.3?}  ({} entries)\n",
        cold.elapsed(),
        report.changed.len()
    );

    let content_ok = measure_targeted_edit(&content, &out, &store, &template)?
        .report("content edit → page on disk", Duration::from_millis(50));
    // Reported for information: the cold path a watcher never takes.
    measure_content_rescan(&content, &out, &store, &template)?
        .report("(full rescan of every file)", Duration::from_millis(50));
    let template_ok = measure_template_edit(&out, &store, &keys)?.report(
        "template markup edit → page on disk",
        Duration::from_millis(100),
    );

    println!();
    Ok(content_ok && template_ok)
}

/// Criterion 1: edit one post; the watcher hands us the path, so no directory is rescanned.
fn measure_targeted_edit(
    content: &Path,
    out: &Path,
    store: &Store,
    template: &Template,
) -> Result<Measurements, Box<dyn std::error::Error>> {
    let mut samples = Vec::with_capacity(ITERATIONS);
    let target = content.join("post-0042.md");
    let original = std::fs::read_to_string(&target)?;

    for iteration in 0..ITERATIONS {
        let edited = original.replace("harbour", &format!("harbour-{iteration}"));
        std::fs::write(&target, &edited)?;

        let start = Instant::now();
        let mut rendered = false;
        if let Some(key) = tri_content::load_file(content, &target, store)?
            && let Some(post) = store.get::<Post>(&key)?
        {
            write_page(out, &post, template)?;
            rendered = true;
        }
        samples.push(start.elapsed());

        // A benchmark that silently measures a no-op is worse than no benchmark. The rescan loop
        // asserted its work; this one did not, until the M0 review.
        assert!(
            rendered,
            "iteration {iteration} did no work: the edit did not change the file's digest"
        );
    }

    std::fs::write(&target, original)?;
    Ok(Measurements { samples })
}

/// The pessimistic path: rescan every file, hash it, and compare.
fn measure_content_rescan(
    content: &Path,
    out: &Path,
    store: &Store,
    template: &Template,
) -> Result<Measurements, Box<dyn std::error::Error>> {
    let mut samples = Vec::with_capacity(RESCAN_ITERATIONS);
    let target = content.join("post-0042.md");
    let original = std::fs::read_to_string(&target)?;

    for iteration in 0..RESCAN_ITERATIONS {
        let edited = original.replace("harbour", &format!("harbour-{iteration}"));
        std::fs::write(&target, &edited)?;

        let start = Instant::now();
        let report = tri_content::load_dir(content, store)?;
        for key in &report.changed {
            if let Some(post) = store.get::<Post>(key)? {
                write_page(out, &post, template)?;
            }
        }
        samples.push(start.elapsed());

        assert_eq!(
            report.changed.len(),
            1,
            "exactly one post should have moved"
        );
    }

    std::fs::write(&target, original)?;
    Ok(Measurements { samples })
}

/// Criterion 2: edit the template's markup, re-parse, swap the descriptor, re-render.
fn measure_template_edit(
    out: &Path,
    store: &Store,
    keys: &[String],
) -> Result<Measurements, Box<dyn std::error::Error>> {
    let mut samples = Vec::with_capacity(ITERATIONS);
    let key = keys.first().ok_or("fixture produced no posts")?;
    let post = store.get::<Post>(key)?.ok_or("missing post")?;
    let mut current = lower(fixture::PAGE_TEMPLATE);

    for iteration in 0..ITERATIONS {
        // Markup only: a wrapper element. The expression table is untouched, which is precisely
        // the condition that makes a descriptor swap legal.
        let edited = fixture::PAGE_TEMPLATE.replace(
            "<article>",
            &format!("<article class=\"v{iteration}\"><div class=\"inner\">"),
        );
        let edited = edited.replace("</article>", "</div></article>");

        let start = Instant::now();
        let parsed = tri_compiler::parse(&edited)?;
        let next = tri_compiler::descriptor::lower(&parsed);
        let swappable = current.is_hot_swappable_with(&next);
        assert!(swappable, "markup-only edit must not require a rebuild");
        write_page(out, &post, &next)?;
        samples.push(start.elapsed());

        current = next;
    }

    Ok(Measurements { samples })
}

fn lower(source: &str) -> Template {
    let parsed = tri_compiler::parse(source).unwrap_or_default();
    tri_compiler::descriptor::lower(&parsed)
}

fn write_page(
    out: &Path,
    post: &Post,
    template: &Template,
) -> Result<(), Box<dyn std::error::Error>> {
    let bindings = PostPage {
        post,
        expressions: &template.expressions,
    };
    let mut sink = StringSink::with_capacity(8 * 1024);
    tri_core::render(template, &bindings, &mut sink)?;
    let dir = out.join("blog").join(&post.slug);
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join("index.html"), sink.into_string())?;
    Ok(())
}

fn write_index(
    out: &Path,
    store: &Store,
    keys: &[String],
    template: &Template,
) -> Result<(), Box<dyn std::error::Error>> {
    let posts: Vec<Post> = keys
        .iter()
        .filter_map(|k| store.get::<Post>(k).ok().flatten())
        .collect();
    let bindings = IndexPage {
        title: "Triblenka fixture",
        posts: posts
            .iter()
            .map(|post| PostPage {
                post,
                expressions: &template.expressions,
            })
            .collect(),
        expressions: &template.expressions,
    };
    let mut sink = StringSink::with_capacity(64 * 1024);
    tri_core::render(template, &bindings, &mut sink)?;
    std::fs::write(out.join("index.html"), sink.into_string())?;
    Ok(())
}

/// Where the harness writes. Exposed so `tri fixture` can point at the same place.
#[must_use]
pub fn default_root() -> PathBuf {
    std::env::temp_dir().join("tri-m0-bench")
}
