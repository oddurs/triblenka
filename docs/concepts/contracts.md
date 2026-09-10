# Contracts

> Design-stage docs.

Every framework promises performance and accessibility. Almost none of them enforce it, so the
promise decays into a slogan and your bundle grows 40 KB a quarter.

A Triblenka **contract** is a property a page or a site declares, and the build refuses to produce
output that violates it. Not a lint you can ignore, not a score in a report you look at quarterly:
a compile error naming the component that broke it.

```rust
#![deny(javascript)]          // no byte of JS may reach this page
#![require(no_js_fallback)]   // every interaction must work with JavaScript disabled
#![deny(external_requests)]   // no third-party origin on the critical path
#![require(alt_text)]         // every image has meaningful alt text
```

Declare them per page, in the frontmatter, or site-wide in `src/site.rs`.

## `deny(javascript)`

The strictest one, and the reason the framework's default is zero JavaScript rather than "not much"
JavaScript.

```
error: `Newsletter` would ship JavaScript to a page that denies it
  ┌─ src/pages/about.tri:14:3
  │
14 │   <Newsletter rung:island />
  │    ^^^^^^^^^^ this island costs 1.1 KB JS + 44 KB wasm
  │
  = the contract is declared at src/pages/about.tri:2
  = help: `Newsletter` is a form; a server frame (rung 1) would satisfy the contract.
          See docs/concepts/interactivity.md
```

Documentation sites, marketing pages, legal and archival pages have no business shipping a runtime.
Now they cannot.

## `require(no_js_fallback)`

Progressive enhancement, checked rather than remembered.

Rungs 0, 1 and 2 of the [interactivity ladder](interactivity.md) all have a working no-JavaScript
path *by construction* — a frame is a link or a form first and an enhancement second — so the
compiler can prove the property from the rung, without running a browser with JavaScript disabled.
A rung-3 island cannot satisfy it, and the build says which component and why.

This is the contract worth adopting first if you adopt only one. It is the difference between a site
that degrades and a site that goes blank.

## `deny(external_requests)`

No third-party origin on the critical path — no font CDN, no analytics beacon, no embedded widget
that hangs the render. The [font pipeline](../guides/assets.md) self-hosts, so the good path is the
easy one. Violations name the URL and the file it came from.

## Accessibility contracts

Some checks are useful enough to be on by default (`alt` is a required prop, not a lint). Others are
opt-in because retrofitting them to an existing site is work:

| Contract | Checks |
|---|---|
| `require(alt_text)` | every image has non-empty, non-redundant alt text |
| `require(heading_order)` | no skipped levels; exactly one `h1` per page |
| `require(labels)` | every form control has an associated label |
| `require(lang)` | `<html lang>` present, and correct per locale |

These are build-time checks over the rendered HTML, which is possible because the build already
produces every page and knows its provenance. They cost a pass over output that already exists.

## Budgets

The numeric members of the same family, declared in `tri.toml`:

```toml
[budgets]
js_per_page   = "5kb"
wasm_per_page = "60kb"
css_per_page  = "20kb"
```

`tri check --budgets` fails the build, in CI, with the route and the overage. A budget that only
warns is a budget that is already broken.

## Why contracts rather than lints

A lint is advice. A contract is a decision, made once, by the person who cares, and enforced
afterwards on everyone — including the version of you that is in a hurry six months from now.

The cost is real and worth stating: contracts make some changes fail that would otherwise merely be
regrettable. That is the point. If a page has to ship a runtime, delete the contract deliberately in
a reviewed diff, rather than letting the property erode one convenient exception at a time.
