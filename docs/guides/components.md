# Components

> Design-stage docs.

A `.tri` file is a server component: Rust frontmatter, then markup. It runs on the server,
produces HTML, and ships no JavaScript.

Interactivity is not declared here. If a component needs to do something in the browser, the
compiler infers which rung of the [interactivity ladder](../concepts/interactivity.md) it needs —
usually a server frame rather than an island — and you override with `rung:*` only when you
disagree.

```html
---
// frontmatter: Rust
---
<!-- template: markup -->
```

## Props

Declare a `Props` struct in the frontmatter. Attributes on the tag map to its fields by name.

```html
---
// src/components/card.tri
use triblenka::prelude::*;

#[props]
fn (title: &str, href: Route, featured: bool = false);
---

<article class="card" class:featured={featured}>
  <h3><a href={href}>{ title }</a></h3>
  <slot />
</article>
```

```html
<Card title="Hello" href={route!(blog::post("hello"))} featured>
  <p>Any markup here fills the default slot.</p>
</Card>
```

- A parameter without a default is required; omitting it is a compile error pointing at the tag.
- Parameters are bound as plain variables in the template — `{ title }`, not `{ props.title }`.
- `{name}` on its own is shorthand for `name={name}`.
- A boolean attribute with no value means `true`.
- Props are ordinary Rust types, so lifetimes, generics, and borrows all work.

For a prop set shared by several components, declare a struct instead and spread it:

```rust
#[props] pub struct CardProps<'a> { pub title: &'a str, pub href: Route }
```

```html
<Card ..card_props />
```

## Interpolation and escaping

```html
<p>{ post.title }</p>              <!-- HTML-escaped -->
<p>{ Html(post.body_html) }</p>    <!-- raw, explicitly opted in -->
```

`{ expr }` requires `expr: impl Render`. Strings, numbers, `Option<T>` (renders nothing for
`None`), and anything implementing `Display` via the `Render` blanket impl are accepted. Raw HTML
must be wrapped in the `Html` newtype — greppable and impossible to do by accident.

## Control flow

Five block forms. Everything inside the parentheses is real Rust, type-checked by `rustc`.

```html
{#if user.is_admin()}
  <AdminBar />
{:else if user.is_member()}
  <MemberBar />
{:else}
  <a href={route!(login())}>Sign in</a>
{/if}

{#for (i, post) in posts.iter().enumerate()}
  <li>{ i + 1 }. { post.title }</li>
{:empty}
  <li>Nothing published yet.</li>
{/for}

{#match status}
  {:case Status::Draft}   <Badge tone="muted">Draft</Badge>
  {:case Status::Live}    <Badge tone="ok">Live</Badge>
{/match}

{#let excerpt = post.body.excerpt(160)}
  <p>{ excerpt }</p>
{/let}

{#if let Some(image) = &post.hero}
  <Image src={image} width={1200} alt={&post.title} />
{/if}
```

If a template needs more logic than this, put it in the frontmatter or a helper function. That is
the intended pressure.

## Slots

```html
<!-- src/layouts/base.tri -->
<html lang="en">
  <head>
    <title>{ props.title }</title>
    <slot name="head" />
  </head>
  <body>
    <slot />                        <!-- default slot -->
    <footer><slot name="footer">© 2026</slot></footer>   <!-- fallback content -->
  </body>
</html>
```

```html
<Base title="Post">
  <meta slot="head" name="description" content={&post.description} />
  <article>…</article>
  <p slot="footer">Written by { post.author.name }</p>
</Base>
```

Slots are closures, not strings: nothing is buffered and no intermediate allocation happens.

## Async frontmatter

Frontmatter may `await`. The compiler notices and makes the component's render async; callers
need no annotation.

```html
---
let rates = fetch_json::<Rates>("https://api.example.com/rates").await?;
---
<p>1 ISK = { rates.usd } USD</p>
```

In `output = "static"` this runs at build time. `tri check` warns when a prerendered page performs
network I/O, since it silently couples your build to a third party.

## Errors

Frontmatter returns `Result`. `?` propagates into a render error, which in dev shows an overlay
with the `.tri` source and span, and in production returns your configured 500 page.

## Fragments and raw blocks

```html
<>                         <!-- fragment: no wrapper element -->
  <dt>{ term }</dt>
  <dd>{ def }</dd>
</>

<pre is:raw>{ this is not interpolated }</pre>
```

## Inline components in `.rs` files

For small components — or when you want proc-macro-perfect spans:

```rust
triblenka::component! {
    pub fn Badge(tone: Tone, children: Slot) {
        <span class={format!("badge badge-{tone}")}><slot /></span>
    }
}
```

Same grammar, same output. `.tri` files are preferred for anything a designer will touch.
