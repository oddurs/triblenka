# Publishing: content deploys and code deploys

> Design-stage docs. This guide describes the workflow the framework is designed around — see
> [Why Triblenka exists](../concepts/why.md).

Triblenka splits publishing into two operations with different cadences, different risk, and
different approval paths.

| | Code deploy | Content deploy |
|---|---|---|
| Triggered by | a merged PR | a write to content or a CMS webhook |
| Rebuilds | the binary | nothing — the binary is unchanged |
| Latency | a CI build | milliseconds |
| Blast radius | the whole site | the pages that read the changed fields |
| Rollback | redeploy the previous binary | restore the previous store |

## Content deploys

```
  edit / webhook  ──►  loader  ──►  store write  ──►  re-render affected  ──►  upload + purge
                                        │
                                        └─ digest unchanged? stop here.
```

```sh
tri content pull            # run loaders, update the store
tri build --changed         # render only what the store write affected
tri deploy --plan           # show the upload + purge plan
tri deploy                  # execute it
```

`tri build --changed` uses the same graph as [`tri impact`](../concepts/incrementality.md): the set
of changed outputs is the upload list, and their URLs are the purge list. A typo fix uploads one
file and purges one URL.

With `output = "server"`, there is nothing to upload — the running binary reads the new store and
serves the change on the next request.

### From a CMS

```rust
// src/pages/api/webhook.rs
pub async fn post(mut cx: Context) -> Result<Response> {
    let event: CmsEvent = cx.json().await?;
    cx.verify_signature(secret!("CMS_WEBHOOK_SECRET"))?;

    let changed = content::refresh("blog", RefreshContext::from(event)).await?;
    if changed.is_empty() {
        return Ok(Response::no_content());
    }
    revalidate::entries(&changed).await?;
    Ok(Response::json(&changed))
}
```

The loader receives the webhook payload as `cx.refresh`, so it can fetch one entry instead of
re-syncing a whole collection. Publish latency is a store write plus a render.

## Code deploys

Normal software: merge a PR, CI builds the binary, the binary ships. The store is not rebuilt and
content is not re-fetched.

**The store is version-checked against the binary.** Every store records the hash of the collection
schemas that wrote it. A binary refuses to read a store written under a different schema, with a
message naming the collection and the change:

```
error: store schema mismatch for collection `blog`
  = the store was written with a schema hashed 3f9a…, this binary expects 7c21…
  = field `subtitle: String` was added at src/content.rs:14
  = help: run `tri content migrate` to bring the store forward, or `tri content pull` to reload
```

This is the failure mode the two-artifact model has to get right: a code deploy that changes a
schema without migrating the store must fail loudly at startup rather than serve half-broken pages.

## Schema migrations

Renaming a frontmatter field across four thousand markdown files is the kind of chore that never
happens, so schemas rot. Migrations make it a normal operation:

```rust
// migrations/0002_split_author.rs
migration!("split author into a reference", |m: &mut Migration<Post>| {
    m.rename("author_name", "author")?;
    m.map("author", |v: String| Ref::<Author>::by_name(&v))?;
    Ok(())
});
```

```sh
tri content migrate --dry-run     # per-entry diff, nothing written
tri content migrate               # apply to the store
tri content migrate --write-files # also rewrite content/**, so the source matches
```

`--dry-run` is the default in CI. Migrations are ordinary code, reviewed in a PR like everything
else, and the store records which have been applied.

## Rollback

- **Code:** redeploy the previous binary. If it expects an older schema, it refuses to start rather
  than misread the store — that refusal is the feature.
- **Content:** the store keeps the previous generation. `tri content rollback` restores it and
  re-renders. For static output, the deploy plan is inverted and replayed.

## Consistency guarantees, honestly

- A store write is atomic; a render either sees the old entry or the new one, never a torn one.
- Uploads are not transactional against a CDN. Between the first and last file of a multi-page
  change, a reader can see a mixed state. For most content sites this is invisible; if it matters,
  deploy to a versioned prefix and flip an alias.
- Multi-instance server deployments need shared store storage or a store push to each instance.
  `tri deploy` handles the adapters that support it and tells you plainly when it cannot.
