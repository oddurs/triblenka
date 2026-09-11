//! Probes written during the M0 review. Each asserts the behaviour that exists *today*, so a fix
//! makes the probe fail loudly rather than silently changing meaning.

use std::path::PathBuf;
use tri_content::{Post, Store, load_dir};

fn fixture(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("tri-review-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("creates");
    dir
}

const POST: &str = "---\ntitle: Hello\ndate: 2026-09-10\ndescription: A greeting\n---\n\nBody.\n";

#[test]
fn the_same_stem_in_two_directories_stays_distinct() {
    let dir = fixture("collide");
    std::fs::create_dir_all(dir.join("a")).expect("creates");
    std::fs::create_dir_all(dir.join("b")).expect("creates");
    std::fs::write(dir.join("a/post.md"), POST.replace("Hello", "From A")).expect("writes");
    std::fs::write(dir.join("b/post.md"), POST.replace("Hello", "From B")).expect("writes");

    let store = Store::open(&dir.join("store.redb")).expect("opens");
    let report = load_dir(&dir, &store).expect("loads");

    assert_eq!(
        store.keys().expect("keys"),
        vec!["blog/a/post".to_owned(), "blog/b/post".to_owned()]
    );
    let a: Post = store.get("blog/a/post").expect("get").expect("present");
    let b: Post = store.get("blog/b/post").expect("get").expect("present");
    assert_eq!((a.title.as_str(), b.title.as_str()), ("From A", "From B"));
    assert_eq!(report.changed.len(), 2);
}

#[test]
fn known_gap_deleted_files_leave_their_entries_behind() {
    let dir = fixture("delete");
    std::fs::write(dir.join("gone.md"), POST).expect("writes");
    let store = Store::open(&dir.join("store.redb")).expect("opens");
    load_dir(&dir, &store).expect("loads");
    assert_eq!(store.keys().expect("keys"), vec!["blog/gone".to_owned()]);

    std::fs::remove_file(dir.join("gone.md")).expect("removes");
    load_dir(&dir, &store).expect("loads");

    assert_eq!(
        store.keys().expect("keys"),
        vec!["blog/gone".to_owned()],
        "TODO: a deleted source file should delete its entry"
    );
}
