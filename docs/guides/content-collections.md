# Content collections

> Design-stage docs.

Content lives in `content/` and is **never compiled**. It is loaded into a digest-keyed store and
queried through typed Rust structs. This is why editing a blog post rebuilds in milliseconds while
editing a template invokes `rustc`.

## Defining a collection

```rust
// src/content.rs
use triblenka::prelude::*;

#[derive(Collection, Deserialize)]
#[collection(name = "blog", loader = Glob::new("content/blog/**/*.md"))]
pub struct Post {
    pub title: String,
    pub date: Date,
    pub description: String,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    pub author: Ref<Author>,       // must resolve to an entry in `authors`
    pub hero: Option<ImageRef>,    // participates in the image pipeline
    #[content]
    pub body: Markdown,            // the document body below the frontmatter
}

#[derive(Collection, Deserialize)]
#[collection(name = "authors", loader = Glob::new("content/authors/*.yaml"))]
pub struct Author {
    pub name: String,
    pub url: Option<Url>,
}
```

Register them:

```rust
// src/site.rs
Site::new().collection::<Post>().collection::<Author>()
```

The derive gives you deserialization, a digest function, a JSON Schema (for editors and CMS
round-tripping), and a typed query surface.

### Validation is a build error

```
error: invalid frontmatter in collection `blog`
  ┌─ content/blog/hello.md:4:1
  │
4 │ date: yesterday
  │ ^^^^^^^^^^^^^^^ expected a date in `YYYY-MM-DD` form
  │
  = help: field `date: Date` is declared at src/content.rs:9
```

A `Ref<Author>` that points nowhere fails the same way. Nothing reaches production as `undefined`.

## Querying

```rust
content::blog()                       // Query<Post>
    .published()                      // !draft, and date <= now
    .by_tag("rust")
    .sorted_by_key(|p| Reverse(p.date))
    .take(10)

content::blog().by_slug("hello-world")   // Option<&Post>
content::authors().get("oddur")          // Option<&Author>
post.author.resolve()                    // &Author, no lookup boilerplate
```

`Query<T>` is an `Iterator`, so the whole standard library is available. Entries are borrowed
from the in-memory index — queries do not allocate or copy documents.

Pagination:

```rust
let page = content::blog().published().paginate(12, params.page)?;
// page.items, page.current, page.total, page.prev_url, page.next_url
```

## Markdown

`Markdown` fields are parsed at load time with [comrak](https://github.com/kivikakk/comrak):

```rust
post.body.html()          // Html, ready to render
post.body.excerpt(160)    // plain-text excerpt
post.body.toc()           // TableOfContents, headings with slugs
post.body.reading_time()  // Duration
```

Build-time passes: GFM (tables, footnotes, strikethrough, task lists), syntax highlighting,
heading anchors, smart typography, image references routed into the [image
pipeline](assets.md), and internal-link checking (`tri check --strict` fails on a dead one).

Configure in `tri.toml`:

```toml
[markdown]
theme = "catppuccin-mocha"
heading_anchors = true
smartypants = true
```

### Components inside markdown

Register a whitelist and use them in any `content/**/*.md` file:

```rust
Site::new().mdx_component::<Callout>("Callout").mdx_component::<Video>("Video")
```

```markdown
Regular prose.

<Callout tone="warn">Be careful with this one.</Callout>
```

The markdown stays data — it still reloads without `rustc` — and the components are dispatched at
render time. If you need *any* component in scope plus full frontmatter, write the page as code
instead: `src/pages/**/*.md.tri`, which is compiled and unrestricted.

## Loaders

A loader fills the store. `Glob` and `File` cover local content; anything else implements the trait:

```rust
#[async_trait]
impl Loader for LinearIssues {
    fn name(&self) -> &str { "linear" }

    async fn load(&self, cx: &LoaderContext) -> Result<()> {
        let since: Option<String> = cx.meta.get("cursor")?;
        let page = linear::issues_since(since.as_deref()).await?;

        for issue in page.items {
            let digest = cx.digest(&issue);
            cx.store.set(&issue.id, &issue, digest)?;   // no-op if unchanged
        }

        cx.meta.set("cursor", &page.cursor)?;
        Ok(())
    }
}
```

- `cx.store.set` compares digests and returns `Changed::No` for unchanged entries, so only the
  pages whose content actually moved are re-rendered.
- `cx.meta` persists sync cursors and ETags across builds.
- `cx.refresh` carries the webhook payload when a build was triggered by one, so a loader can
  refresh a single entry instead of a whole CMS.

Built-in loaders: `Glob`, `File`, `Http` (with ETag handling), `Sql` (via `sqlx`), `Git`
(last-modified and authorship from history).

## Live collections

For data too fresh to bake in, query at request time in `server` or `hybrid` output:

```rust
let stock = live::inventory().get(&params.sku).await?;
```

Live collections skip the store and hit the loader directly. They are only allowed on routes
that are not prerendered; using one on a prerendered route is a build error naming the route.
