# Errors

> Design-stage docs.

Most of the people who hit a Triblenka error are not Rust programmers. They are writers with a
malformed date, editors with a broken link, designers with a typo'd component name. **Error quality
is a product feature here, not a politeness.**

## The contract

Every error names four things:

1. **Where** — file, line, column, with the source excerpt.
2. **What** — what was found.
3. **What was expected** — and where that expectation is declared.
4. **What to do** — a concrete fix, whenever one can be inferred.

```
error: invalid frontmatter in collection `blog`
  ┌─ content/blog/hello.md:4:7
  │
4 │ date: yesterday
  │       ^^^^^^^^^ expected a date like 2026-09-09
  │
  = the field `date: Date` is declared at src/content.rs:9
  = help: if you meant today, write `date: 2026-09-09`
```

Note what is *not* in there: no Rust types, no serde jargon, no backtrace, no mention of the
generated code that actually failed to compile.

## Generated code is never shown

`.tri` files compile to Rust, so a naive implementation surfaces `rustc` errors pointing at machine
output — the failure mode that makes file-based template engines unpleasant to use. Two mechanisms
prevent it:

- **Pre-flight checks** catch what does not need `rustc` at all: unknown component, missing required
  prop, slot typo, unclosed block, unknown directive, void-element misuse. These are roughly 80% of
  real template errors, and they are reported directly against the `.tri` source.
- **Span remapping** handles the rest. The compiler records the `.tri` span for every emitted byte
  range; `tri build` parses cargo's JSON diagnostics and re-renders them against the original file.

If you ever see a path containing `OUT_DIR` or `tri_generated`, that is a bug — please report it
with the reproduction.

## Examples of the standard

```
error: unknown component `Crad`
  ┌─ src/pages/index.tri:14:4
  │
14 │   <Crad title="Hello" />
  │    ^^^^ not found in scope
  │
  = help: did you mean `Card`? (src/components/card.tri)
  = note: 4 components are in scope here: Base, Card, Prose, Nav
```

```
error: missing required prop `href` on `Card`
  ┌─ src/pages/index.tri:14:4
  │
14 │   <Card title="Hello" />
  │    ^^^^ `href: Route` has no default
  │
  = the prop is declared at src/components/card.tri:5
```

```
error: dead internal link
  ┌─ content/blog/hello.md:22:14
  │
22 │ See the [routing guide](/docs/routing/).
  │              ^^^^^^^^^^^^^^^^ no route matches `/docs/routing/`
  │
  = help: the closest match is `/docs/guides/routing/`
  = note: reported by `tri check --strict`; downgrade with `--no-links`
```

## Warnings must be actionable or absent

A warning nobody acts on trains people to ignore all warnings. Every warning must either suggest a
fix or be suppressible with a documented flag. If neither is possible, it is a note in `--stats`,
not a warning.

## For contributors

- Errors are `miette` diagnostics with spans. An error without a span needs a good reason.
- Write the message for someone who has never read the source of this project.
- No `unwrap`, `expect`, or `panic!` on a path a user's input can reach. A malformed file is an
  error value, not a crash.
- Prefer one precise error over three vague ones — but when several independent problems exist,
  report them all in one run rather than making the user fix them one at a time.
- Every error message that ships gets a test asserting its text. Messages are an interface.
