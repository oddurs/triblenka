# API reference

> Design-stage docs. Signatures are the designed shape and will move before 0.1.

## Rendering

```rust
pub trait Component {
    type Props<'a>;
    const SCOPE: ScopeId;
    const STYLE: &'static str;

    async fn render(props: Self::Props<'_>, slots: Slots<'_>, sink: &mut Sink<'_>) -> Result<()>;
}

pub trait Render {
    fn render_escaped(&self, sink: &mut Sink<'_>) -> Result<()>;
}

pub trait Sink {
    fn raw(&mut self, s: &str) -> Result<()>;
    fn escaped(&mut self, v: impl Render) -> Result<()>;
    fn defer(&mut self, id: IslandId) -> Result<()>;
}
```

Rendering is a depth-first write into a `Sink`. There is no virtual DOM and no intermediate tree:
static markup compiles to `sink.raw(&'static str)` over pre-concatenated literals.

`Html(s)` is the newtype that renders without escaping. `Slots` exposes `default(sink)` and
`named("footer", sink)`, both closures rather than buffered strings.

## Frames and handlers

```rust
pub trait Frame {
    const ID: &'static str;
    type Props<'a>: Serialize + DeserializeOwned;

    /// Render just this fragment. Same code path as rendering it inside a full page.
    async fn render(props: Self::Props<'_>, sink: &mut Sink<'_>) -> Result<()>;

    /// Cache tags, derived from the fields this frame's render actually read.
    fn tags(props: &Self::Props<'_>) -> Vec<CacheTag>;
}

pub trait Handler {
    type Captured: Serialize + DeserializeOwned;
    const CHUNK: ChunkId;
    fn call(captured: Self::Captured, event: Event) -> Result<()>;
}
```

A frame renders through the ordinary `Sink`, so a fragment and a full page share one code path and
cannot drift. `tags()` is generated, not written — see
[incrementality](../concepts/incrementality.md).

## Contracts

```rust
pub enum Contract {
    DenyJavaScript,
    RequireNoJsFallback,
    DenyExternalRequests,
    RequireAltText,
    RequireHeadingOrder,
    RequireLabels,
    RequireLang,
}
```

Declared with inner attributes in a page's frontmatter (`#![deny(javascript)]`) or site-wide in
`src/site.rs`. Enforcement runs over rendered output and names the component that broke the
contract. Rungs 0–2 satisfy `RequireNoJsFallback` by construction, so it is proved from the rung
rather than tested in a browser. See [Contracts](../concepts/contracts.md).

## Content

```rust
pub trait Collection: DeserializeOwned + Send + Sync + 'static {
    const NAME: &'static str;
    fn loader() -> Box<dyn Loader>;
    fn digest(&self) -> Digest;
    fn schema() -> Schema;
}

#[async_trait]
pub trait Loader: Send + Sync {
    fn name(&self) -> &str;
    async fn load(&self, cx: &LoaderContext) -> Result<()>;
    fn watch(&self) -> Vec<Pattern> { vec![] }
}

pub struct LoaderContext {
    pub store: Store,
    pub meta: MetaStore,
    pub logger: Logger,
    pub config: Arc<Config>,
    pub refresh: Option<RefreshData>,
}

impl Store {
    pub fn set<T: Serialize>(&self, id: &str, v: &T, d: Digest) -> Result<Changed>;
    pub fn get<T: DeserializeOwned>(&self, id: &str) -> Result<Option<T>>;
    pub fn delete(&self, id: &str) -> Result<()>;
    pub fn keys(&self) -> impl Iterator<Item = String>;
}
```

`Changed::No` means the digest matched and nothing downstream is invalidated — the mechanism that
keeps a full CMS refetch from rebuilding a whole site.

## Islands

```rust
#[island]                       // attribute macro
pub fn Counter(start: i32) -> impl IntoView;

pub trait IslandRenderer {
    const NAME: &'static str;
    type Component<P>;
    fn render_to_sink<P: Props>(c: &Self::Component<P>, p: &P, s: &mut Sink) -> Result<()>;
    fn client_entry() -> ClientEntry;
}

pub trait Props: Serialize + DeserializeOwned + Send + 'static {}
```

## Adapters

```rust
pub trait Adapter {
    fn name(&self) -> &str;
    fn features(&self) -> Features;
    fn finalize(&self, out: &BuildOutput) -> Result<()>;
}

pub struct Features {
    pub streaming: Support,        // Stable | Limited | Unsupported
    pub server_islands: Support,
    pub on_demand_images: Support,
    pub edge_middleware: Support,
    pub secrets: Support,
    pub cache_tags: Support,
}
```

The runtime half of every adapter wraps one shared service:

```rust
impl tower::Service<http::Request<Body>> for App {
    type Response = http::Response<Body>;
}
```

## Integrations

```rust
#[async_trait]
pub trait Integration: Send + Sync {
    fn name(&self) -> &str;
    async fn config_setup(&self, cx: &mut ConfigCtx) -> Result<()> { Ok(()) }
    async fn routes_resolved(&self, cx: &mut RouteCtx) -> Result<()> { Ok(()) }
    async fn page_rendered(&self, cx: &mut PageCtx<'_>) -> Result<()> { Ok(()) }
    async fn build_done(&self, cx: &BuildCtx) -> Result<()> { Ok(()) }
}
```

First-party: `sitemap`, `rss`, `og_image`, `search`, `redirects`, `robots`, `tailwind`, `sass`,
`check_links`, `analytics`.

## Macros

| Macro | Purpose |
|---|---|
| `route!(blog::post(&slug))` | typed URL for a page; compile error if the route is gone |
| `asset!("images/hero.jpg")` | fingerprinted asset URL; compile error if missing |
| `secret!("STRIPE_KEY")` | runtime secret, resolved by the adapter |
| `classes![…]` | class-list builder accepting `&str` and `Option<&str>` |
| `content::<name>()` | generated per collection; returns `Query<T>` |
| `triblenka::component! { … }` | inline component in a `.rs` file |
| `use_style!("styles/global.css")` | attach a stylesheet to the current component |

## Errors

```rust
pub enum Error {
    NotFound,
    Content { collection: &'static str, id: String, source: ContentError },
    Render { component: &'static str, span: Span, source: BoxError },
    Adapter(BoxError),
    Other(anyhow::Error),
}
```

All errors carry spans where they have them; the dev overlay and CLI render them through
[`miette`](https://docs.rs/miette) against the original `.tri` or markdown source, never against
generated code.
