# Images and assets

> Design-stage docs.

## Images

```html
---
use triblenka::prelude::*;
---
<Image src={asset!("images/hero.jpg")} width={1200} alt="The harbour at dusk" />
```

At build time Triblenka decodes, resizes (Lanczos3), and re-encodes to AVIF and WebP with an
original-format fallback, then emits:

```html
<picture>
  <source type="image/avif" srcset="/_tri/hero.a91f.avif 1200w, /_tri/hero.7b2c.avif 600w" sizes="…">
  <source type="image/webp" srcset="…">
  <img src="/_tri/hero.3d1e.jpg" width="1200" height="675" alt="The harbour at dusk"
       loading="lazy" decoding="async" style="background-image:url(data:…)">
</picture>
```

`width` and `height` are always emitted so layout never shifts. `alt` is required — omitting it is
a compile error, not a lint. A tiny blur placeholder is inlined as a data URL.

```html
<Image src={…} width={1200} priority alt="…" />   <!-- eager + fetchpriority=high + preload -->
<Image src={…} widths={[400, 800, 1200]} sizes="(min-width: 60rem) 50vw, 100vw" alt="…" />
```

Images referenced from markdown frontmatter (`ImageRef`) and from markdown bodies go through the
same pipeline.

### Remote images

```toml
[images]
domains = ["images.unsplash.com"]
formats = ["avif", "webp"]
quality = 78
```

Remote images are downloaded and processed at build time; in `server` output, `/_tri/image` serves
them on demand with signed parameters and a disk cache. Undeclared domains are rejected, so your
image endpoint cannot be used as an open proxy.

## Fonts

```toml
[[fonts]]
family  = "Inter"
weights = [400, 600]
subsets = ["latin", "latin-ext"]
display = "swap"
```

Fonts are downloaded, subset, self-hosted, fingerprinted, and given `@font-face` rules plus
`<link rel=preload>` for the faces actually used above the fold. No third-party font CDN sits on
your critical path.

## Static files

Anything in `public/` is copied to the output root verbatim: `public/favicon.svg` → `/favicon.svg`.
Use it for files that must keep a stable URL — `robots.txt`, verification files, downloads.

For anything else, prefer `asset!("path")`, which fingerprints the file, returns its final URL, and
fails the build if it does not exist.

## Fingerprinting and caching

Every generated asset is content-hashed. The recommended cache headers, which adapters emit
automatically:

| Path | Cache-Control |
|---|---|
| `/_tri/**` | `public, max-age=31536000, immutable` |
| HTML | `public, max-age=0, must-revalidate` |
| `public/**` | `public, max-age=3600` |

Unchanged outputs keep their previous bytes and are skipped on write, so incremental deploys and
CDN invalidations stay small.
