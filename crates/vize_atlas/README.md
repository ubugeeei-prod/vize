# vize_atlas

Compatibility follows the [Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

Source Atlas is Vize's representation-independent, demand-driven artifact graph.
It is the common execution substrate on which compilers, linters, typecheckers,
language-server features, and inspectors can request typed products.

Atlas owns infrastructure, not compiler representations:

- `SourceStore` assigns stable `SourceId`s and monotonic revisions, including
  explicit parent revision/range provenance for embedded sources.
- `Product` and `Provider` are open traits. A product and any alternative
  providers are added in their owning crates; there is no Atlas domain enum or
  central frontend switch to edit.
- Multiple providers can target one product and declare applicability through
  `Provider::supports`. A plan captures the exact concrete `ProviderId`; zero
  or multiple applicable providers are structured planning errors.
- `Provider::dependencies` receives a `PlanningContext`, so dependency closure
  can differ by source shape and typed compilation inputs. JSX does not need to
  plan SFC products, and SFC does not need to plan JSX products.
- `ProductRequest` is the open `(SourceId, ProductId)` graph-node identity.
  `Provider::dependency_requests` and `ProviderContext::get_for_source` support
  imports and project-wide aggregation while the original `dependencies` and
  `get` methods remain concise same-source defaults.
- `CompilationInput` is an open typed store for dialects, target capabilities,
  or project configuration. Planning and execution read the same input value.
- `Plan` contains only requested roots and their transitive dependencies.
- `ArtifactCache` memoizes shared dependencies once per source revision and is
  inspectable through `Compilation::cache`.
- `Product::Value` is owned `'static` cache storage. Products that need a
  borrowed, arena/interned, or iterator-like consumer interface opt into
  `ProductView`; storage may be a cloned `Shared` handle to compilation-owned
  source or arena bytes, so the view does not require copying or materializing
  a collection. Existing products keep using `Product` unchanged.
- `Product::CACHE_POLICY` is memoized by default. A transient product is shared
  inside one multi-root execution but creates no persistent cache entry. Atlas
  also prunes it when every consumer is already cached.
- Cross-source cache entries capture every transitive source revision. Editing
  one participating source evicts its project consumers without discarding
  unrelated source trees.
- providers declare their typed `CompilationInput` dependencies. Each input has
  an independent revision, so an input update evicts only affected products
  (including transitive consumers) and only relevant plans become stale.
- source/input updates return observable invalidation reports; stale plans are
  rejected without discarding unrelated work.
- `ExecutionOutcome`, `ExecutionTrace`, and `ExecutionCounters` expose what was
  queried, executed, or served from cache.
- structured diagnostics, fallback records, and notes are cached with their
  product and attributed to the exact request, provider, source, and range.
- `CompilationSnapshot` captures sources, provenance, inputs, providers, and
  cache as a cheaply cloned immutable editor snapshot; `fork` creates an
  isolated mutable query branch.

Syntax, semantic, control-flow, render, virtual-source, diagnostics, and emitted
artifacts are all peer products implemented outside Atlas. None is the
mandatory center of the graph.

The rejected request-ledger API (`SourceAtlasRegistry`, route, plate, target,
and global fallback enums) is intentionally absent. Observability comes from
the graph's actual plan, provider outcomes, counters, and trace.

## Cost contract

- planning never invokes a provider;
- unselected alternative providers never plan dependencies or execute;
- a provider outside the requested dependency closure executes zero times;
- two roots sharing a dependency execute that dependency once;
- a cached product is reused until its source tree or one of its declared typed
  compilation inputs changes.
- project products are recomputed when any transitive source changes, while
  cached products for unrelated sources survive.

These are properties of compiler execution itself, not generated runtime code.
They are enforced by integration tests in this crate.
