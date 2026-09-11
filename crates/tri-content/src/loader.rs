use crate::post::{Frontmatter, Post};
use crate::store::{Changed, Digest, Store};
use crate::{Error, Result};
use std::path::Path;

/// What one pass of the loader did.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct LoadReport {
    /// Keys whose content actually moved, sorted.
    pub changed: Vec<String>,
    /// How many entries were already up to date.
    pub unchanged: usize,
}

/// Load every `*.md` under `dir` into `store`.
///
/// Files whose digest is unchanged are not parsed, not rendered, and not written — the short
/// circuit the content-rebuild budget depends on.
///
/// # Errors
///
/// Fails on unreadable files, malformed frontmatter, or a store write failure.
pub fn load_dir(dir: &Path, store: &Store) -> Result<LoadReport> {
    let mut report = LoadReport::default();
    // One transaction for every digest, rather than one per file.
    let known = store.all_digests()?;

    // Sorted, because directory order is not stable across platforms
    // (`docs/concepts/determinism.md`, rule 7).
    let mut paths: Vec<_> = walkdir::WalkDir::new(dir)
        .sort_by_file_name()
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .filter(|p| p.extension().is_some_and(|e| e == "md"))
        .collect();
    paths.sort();

    for path in paths {
        let bytes = std::fs::read(&path)?;
        let digest = Digest::of(&bytes);
        let slug = slug_for(dir, &path);
        let key = format!("blog/{slug}");

        if known.get(&key) == Some(&digest) {
            report.unchanged += 1;
            continue;
        }

        let source = String::from_utf8_lossy(&bytes);
        let post = parse_post(&path.display().to_string(), &slug, &source)?;
        if store.set(&key, &post, &digest)? == Changed::Yes {
            report.changed.push(key);
        } else {
            report.unchanged += 1;
        }
    }

    report.changed.sort();
    Ok(report)
}

/// Load a single file, the way a watcher event would.
///
/// The dev loop knows which file changed, so it never needs to rescan a directory. Returns the
/// key if the content actually moved.
///
/// # Errors
///
/// Fails on an unreadable file, malformed frontmatter, or a store write failure.
pub fn load_file(root: &Path, path: &Path, store: &Store) -> Result<Option<String>> {
    let bytes = std::fs::read(path)?;
    let digest = Digest::of(&bytes);
    let slug = slug_for(root, path);
    let key = format!("blog/{slug}");

    if store.digest_of(&key)?.as_ref() == Some(&digest) {
        return Ok(None);
    }

    let source = String::from_utf8_lossy(&bytes);
    let post = parse_post(&path.display().to_string(), &slug, &source)?;
    match store.set(&key, &post, &digest)? {
        Changed::Yes => Ok(Some(key)),
        Changed::No => Ok(None),
    }
}

/// The slug for `path`, derived from its location under `root`.
///
/// Path-derived rather than stem-derived: two files with the same stem in different directories are
/// different documents, and collapsing them silently loses one (found in the M0 review).
fn slug_for(root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path);
    let without_extension = relative.with_extension("");
    let mut parts: Vec<String> = Vec::new();
    for component in without_extension.components() {
        if let std::path::Component::Normal(part) = component {
            parts.push(part.to_string_lossy().into_owned());
        }
    }
    if parts.is_empty() {
        "untitled".to_owned()
    } else {
        parts.join("/")
    }
}

fn parse_post(path: &str, slug: &str, source: &str) -> Result<Post> {
    let rest = source
        .strip_prefix("---")
        .ok_or_else(|| Error::Frontmatter {
            path: path.to_owned(),
            message: "expected the file to open with a `---` frontmatter fence".to_owned(),
        })?;
    let (yaml, body) = rest.split_once("\n---").ok_or_else(|| Error::Frontmatter {
        path: path.to_owned(),
        message: "the frontmatter fence is never closed with `---`".to_owned(),
    })?;

    let meta: Frontmatter = serde_yaml_ng::from_str(yaml).map_err(|e| Error::Frontmatter {
        path: path.to_owned(),
        message: format!("{e}"),
    })?;

    let options = comrak::Options::default();
    let body_html = comrak::markdown_to_html(body.trim_start_matches('-').trim_start(), &options);

    Ok(Post {
        slug: slug.to_owned(),
        title: meta.title,
        date: meta.date,
        description: meta.description,
        draft: meta.draft,
        tags: meta.tags,
        body_html,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn write(dir: &Path, name: &str, body: &str) {
        std::fs::write(dir.join(name), body).expect("writes");
    }

    fn fixture(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("tri-loader-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("creates");
        dir
    }

    const POST: &str = "---\ntitle: Hello\ndate: 2026-09-10\ndescription: A greeting\n---\n\n# Hi\n\nSome *body*.\n";

    #[test]
    fn parses_frontmatter_and_renders_markdown() {
        let post = parse_post("hello.md", "hello", POST).expect("parses");
        assert_eq!(post.title, "Hello");
        assert_eq!(post.date, "2026-09-10");
        assert!(post.body_html.contains("<h1>Hi</h1>"), "{}", post.body_html);
        assert!(
            post.body_html.contains("<em>body</em>"),
            "{}",
            post.body_html
        );
    }

    #[test]
    fn missing_fence_is_an_actionable_error() {
        let err = parse_post("bad.md", "bad", "no frontmatter here").expect_err("must fail");
        assert!(
            format!("{err}").contains("`---` frontmatter fence"),
            "{err}"
        );
    }

    #[test]
    fn malformed_field_names_the_file() {
        let src = "---\ntitle: Hello\n---\nbody";
        let err = parse_post("content/blog/x.md", "x", src).expect_err("must fail");
        let message = format!("{err}");
        assert!(message.contains("content/blog/x.md"), "{message}");
        assert!(message.contains("date"), "{message}");
    }

    #[test]
    fn second_load_of_unchanged_content_changes_nothing() {
        let dir = fixture("unchanged");
        write(&dir, "hello.md", POST);
        let store = Store::open(&dir.join("store.redb")).expect("opens");

        let first = load_dir(&dir, &store).expect("loads");
        assert_eq!(first.changed, vec!["blog/hello".to_owned()]);
        assert_eq!(first.unchanged, 0);

        let second = load_dir(&dir, &store).expect("loads");
        assert!(second.changed.is_empty());
        assert_eq!(second.unchanged, 1);
    }

    #[test]
    fn a_targeted_load_touches_only_the_named_file() {
        let dir = fixture("targeted");
        write(&dir, "a.md", POST);
        write(&dir, "b.md", POST);
        let store = Store::open(&dir.join("store.redb")).expect("opens");
        load_dir(&dir, &store).expect("loads");

        // Unchanged: the digest matches, so nothing is parsed or written.
        assert_eq!(
            load_file(&dir, &dir.join("a.md"), &store).expect("loads"),
            None
        );

        write(&dir, "a.md", &POST.replace("Some *body*", "Edited *body*"));
        assert_eq!(
            load_file(&dir, &dir.join("a.md"), &store).expect("loads"),
            Some("blog/a".to_owned())
        );
    }

    #[test]
    fn editing_one_file_reports_only_that_file() {
        let dir = fixture("one-file");
        write(&dir, "a.md", POST);
        write(&dir, "b.md", POST);
        let store = Store::open(&dir.join("store.redb")).expect("opens");
        load_dir(&dir, &store).expect("loads");

        write(&dir, "b.md", &POST.replace("Some *body*", "Edited *body*"));
        let report = load_dir(&dir, &store).expect("loads");
        assert_eq!(report.changed, vec!["blog/b".to_owned()]);
        assert_eq!(report.unchanged, 1);
    }
}
