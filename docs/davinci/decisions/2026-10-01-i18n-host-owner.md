# Host translation ownership

Decision for [#6834](https://github.com/ubugeeei-prod/vize/issues/6834).

The embedded translator, locale selection, message tables, JSON loading and
process-global catalog belong to `vize_carton`. L0 diagnostic identities and
compiler error codes remain portable. The preceding
[message lookup provider](./2026-10-01-message-lookup-provider.md) lets those
codes consume caller-supplied message text without depending on host storage.

Move the 25 catalog and payload files unchanged in a move-only commit, then
integrate the real host consumers. Carton's `LocaleMessages` implements L0's
`MessageLookup`; CLI explain pages and Relief diagnostics bind that provider.
CLI rendering, linting, Patina, Canon and Vitrine use the host locale/catalog
owner directly. Patina and Canon keep their existing public Locale paths.
L0 exports no forwarding path to Carton and keeps its static English messages.

Locale aliases and indexes, registration order, embedded text, English
fallback, unknown-key ownership, placeholder replacement, custom diagnostic
messages and help selection keep their existing behavior. No conversion,
serialization or pipeline stage is added. Carton retains the existing std
string contract for host JSON parsing and unknown keys under a module-scoped
lint expectation. Its normal dependencies include `once_cell` and
`rustc-hash`; L0 still needs both for its remaining portable/native consumers.
Relief and Vitrine declare their actual Carton host dependency. Normal and
build dependencies continue to point from `crates/` into `davinci/` only.

Replay uses `vp node tools/support/levels/move-i18n-host.ts moves`, then
`integrate`, then `check`. Preflight validates module/test ownership, real
provider calls, exact caller imports, manifests and lock edges before any
integration write. Fully integrated replay is idempotent; mixed ownership or
malformed caller/manifest/lock states reject before writing. Source laws allow
only reviewed i18n imports in their actual host files and still reject Carton
storage, arbitrary catalog imports and native reverse dependencies.

The catalog reader follows the host source paths. Existing locale/catalog,
compiler diagnostic and frozen renderer coverage remains authoritative; no
fixture bytes or numeric ceilings change. Source-only consumer inventories
are refreshed. Publish the provider and consumer as a GitHub native Stack and
validate both exact heads and queue candidates before reporting completion.

This slice does not finish #6834. Framework effective configuration placement,
profiler ownership and full platform/no-std isolation remain open.
