# Configuration reference

> Design-stage docs.

Configuration is split deliberately: **data in `tri.toml`**, **code in `src/site.rs`**. Anything a
non-Rust contributor might change lives in TOML; anything that needs a type lives in Rust.

## `tri.toml`

```toml
base_url = "https://example.com"
base     = "/"                  # deploy under a subpath
output   = "static"             # static | server | hybrid
trailing_slash = "ignore"       # always | never | ignore

[build]
out_dir      = "dist"
parallel     = true             # rayon across routes
fail_on_warn = false

[markdown]
theme            = "catppuccin-mocha"
heading_anchors  = true
smartypants      = true
gfm              = true

[css]
inline_threshold = "12kb"
targets          = "defaults and not dead"

[images]
domains  = ["images.unsplash.com"]
formats  = ["avif", "webp"]
quality  = 78
widths   = [400, 800, 1200, 1600]
placeholder = "blur"

[[fonts]]
family  = "Inter"
weights = [400, 600]
subsets = ["latin"]
display = "swap"

[budgets]
js_per_page   = "5kb"
wasm_per_page = "60kb"
css_per_page  = "20kb"

[redirects]
"/old-blog/:slug" = { to = "/blog/:slug", status = 301 }

[secrets]
required = ["STRIPE_KEY", "DATABASE_URL"]   # checked at build, resolved at runtime

[dev]
port = 4321
open = false

[adapter]
allow_fallbacks = false        # error instead of degrading unsupported features
```

## `src/site.rs`

```rust
use triblenka::prelude::*;

pub fn site() -> Site {
    Site::new()
        // content
        .collection::<Post>()
        .collection::<Author>()
        .loader("linear", LinearIssues::new(secret!("LINEAR_KEY")))

        // markdown components (usable from content/**/*.md)
        .mdx_component::<Callout>("Callout")
        .mdx_component::<Video>("Video")

        // integrations
        .integration(sitemap::default())
        .integration(rss::from::<Post>().title("Blog"))
        .integration(og_image::from_template::<templates::OgCard>())
        .integration(search::pagefind())

        // server
        .middleware(crate::middleware::middleware())

        // target
        .adapter(adapters::Cloudflare::new())
}
```

## Cargo

```toml
[dependencies]
triblenka = { version = "0.1", features = ["islands-vanilla", "markdown", "images"] }

[build-dependencies]
triblenka-build = "0.1"

[profile.dev-fast]           # used by `tri dev`
inherits     = "dev"
opt-level    = 0
debug        = 1
incremental  = true

[profile.island]             # used for the wasm island binary
inherits  = "release"
opt-level = "z"
lto       = true
panic     = "abort"
strip     = true
```

| Feature | Enables |
|---|---|
| `islands-vanilla` | TypeScript islands via oxc + rolldown (**default**) |
| `islands-leptos` | Leptos island renderer |
| `islands-dioxus` | Dioxus island renderer |
| `markdown` | comrak pipeline, highlighting, TOC |
| `images` | image/ravif/oxipng pipeline |
| `sidecar` | opt-in Node/Deno SSR for JS components |
