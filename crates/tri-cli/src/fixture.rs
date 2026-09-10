//! The benchmark fixture: a deterministic blog.
//!
//! Generated rather than committed, because 500 files of prose is noise in a diff and a seeded
//! generator gives the same bytes on every machine — which the reproducibility rules require of
//! anything a benchmark depends on.

use std::io;
use std::path::Path;

/// The page template the benchmark renders. Markup plus expressions, no components yet.
pub const PAGE_TEMPLATE: &str = r#"<!doctype html>
<html lang="en">
<head><title>{ post.title }</title><meta name="description" content="{ post.description }"></head>
<body>
<article>
<h1>{ post.title }</h1>
<p class="meta">{ post.date }</p>
{#if post.draft}<p class="draft">Draft</p>{/if}
{ post.body }
</article>
</body>
</html>
"#;

/// The index template, which loops over every post.
pub const INDEX_TEMPLATE: &str = r#"<!doctype html>
<html lang="en">
<head><title>{ site.title }</title></head>
<body>
<h1>{ site.title }</h1>
<ul>
{#for post in posts}<li><a href="/blog/{ post.slug }/">{ post.title }</a> <time>{ post.date }</time></li>{/for}
</ul>
</body>
</html>
"#;

const WORDS: [&str; 24] = [
    "harbour",
    "compiler",
    "descriptor",
    "island",
    "render",
    "content",
    "digest",
    "store",
    "template",
    "frame",
    "ladder",
    "budget",
    "provenance",
    "determinism",
    "cache",
    "adapter",
    "markdown",
    "collection",
    "loader",
    "sink",
    "runtime",
    "wasm",
    "typed",
    "static",
];

/// Deterministic pseudo-random sequence. No `rand` dependency, and identical on every machine.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }

    fn word(&mut self) -> &'static str {
        let index = usize::try_from(self.next()).unwrap_or(0) % WORDS.len();
        WORDS[index]
    }
}

/// Write `count` posts into `dir`, replacing whatever was there.
///
/// # Errors
///
/// Fails if the directory cannot be created or written.
pub fn generate(dir: &Path, count: usize) -> io::Result<()> {
    let _ = std::fs::remove_dir_all(dir);
    std::fs::create_dir_all(dir)?;
    for index in 0..count {
        std::fs::write(dir.join(format!("post-{index:04}.md")), post(index))?;
    }
    Ok(())
}

/// The markdown source of post `index`. Same bytes for the same index, always.
#[must_use]
pub fn post(index: usize) -> String {
    let mut rng = Lcg(index as u64 + 1);
    let title = format!("{} {}", capitalise(rng.word()), rng.word());
    let day = 1 + (index % 28);
    let month = 1 + (index % 12);
    let tags = format!("[{}, {}]", rng.word(), rng.word());

    let mut body = String::with_capacity(2048);
    for section in 0..4 {
        body.push_str(&format!(
            "\n## {} {}\n\n",
            capitalise(rng.word()),
            rng.word()
        ));
        for _ in 0..3 {
            let mut paragraph = String::new();
            for _ in 0..28 {
                paragraph.push_str(rng.word());
                paragraph.push(' ');
            }
            body.push_str(paragraph.trim_end());
            body.push_str(".\n\n");
        }
        if section == 1 {
            body.push_str("```rust\nfn render(sink: &mut dyn Sink) -> Result<()> {\n    sink.raw(\"<p>\")\n}\n```\n\n");
        }
    }

    format!(
        "---\ntitle: {title}\ndate: 2026-{month:02}-{day:02}\ndescription: {} {}\ndraft: false\ntags: {tags}\n---\n{body}",
        capitalise(rng.word()),
        rng.word(),
    )
}

fn capitalise(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn the_same_index_always_produces_the_same_bytes() {
        assert_eq!(post(7), post(7));
        assert_ne!(post(7), post(8));
    }

    #[test]
    fn posts_have_parseable_frontmatter() {
        let source = post(0);
        assert!(source.starts_with("---\ntitle: "), "{source}");
        assert!(source.contains("\n---\n"));
        assert!(source.contains("## "));
    }
}
