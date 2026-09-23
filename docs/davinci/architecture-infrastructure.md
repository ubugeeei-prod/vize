# Davinci — Architecture: shared infrastructure and extension contracts

> [!NOTE]
> Split from [architecture.md](./architecture.md) under the 350-line source
> budget. Same document, same authority; the stage model, priority order,
> and guardrails stay in the parent page.

## Shared infrastructure (what stages have in common)

- **Pass manager** — each product (compile-dom, compile-vapor, compile-ssr,
  lint, typecheck-projection, format) is a declared pipeline of statically-known
  passes, each marked fusable or barrier as above. Debug builds interleave stage
  verifiers; `profile!` spans wrap each pass automatically. No registry of
  trait objects; pipelines are const data. Passes are additionally classified
  SIL-style as **mandatory-diagnostic / mandatory-lowering / optional-
  optimization**: each stage has a raw→canonical transition (type-level) that
  only mandatory passes perform; mandatory passes run at every optimization
  level, are unfusable barriers, and are where user-facing dataflow
  diagnostics attach — dataflow-hungry lint rules become mandatory-diagnostic
  passes over canonical S2/S3, which structurally ends dual diagnostic
  assembly. Only optional passes participate in fusion and the traversal
  budget; optimization tiers scale **budgets, never pass sets** (the Flambda2
  model — no forked pipelines, all tiers emit summary-compatible output).
- **Folio dumps** — the textual format for every stage, named after the folios
  of Leonardo's manuscripts (the existing croquis "VIR" debug dump is absorbed
  as the croquis folio, with a deprecation alias in the inspector payload).
  Snapshot tests pin any stage; the Compiler Inspector and the
  [DevTool](./devtool.md) render the same dumps.
- **One diagnostics channel** — diagnostics carry a `Span`, a stage of origin,
  and structured parts; all rendering (CLI, LSP, JSON, corpus reports) consumes
  the same finished `Vec<Diagnostic>`. This structurally removes the
  two-independent-assembly-paths failure mode in canon. The UX bar is
  **rustc/Elm-grade** (charter #42): span labels, help, fix suggestions, and a
  witness-derived "why" expansion — the assurance witnesses double as the
  explanation data — with i18n (en/ja/zh) across every diagnostic.
- **Node ids + side tables** — cross-stage references and analysis results are
  `NodeId`-keyed tables, not fat nodes and not raw `*mut` traversal.
- **Stage artifact keys** — every stage output has a content-derived identity
  (Doctor's `cache_identity` pattern, promoted), at SFC-block granularity. This
  is the substrate #698 (block-level virtual TS reuse) and #699 (session reuse)
  are waiting for, and what lets Maestro stop re-parsing per request.
- **Two-tier incrementality (decided)** — resident processes (Maestro,
  `check-server`, watch modes) run stages as **salsa** queries keyed by the
  stage artifact identities; one-shot CLI runs (`build`, `fmt`, `lint`) use the
  fused non-salsa pipeline. Same stage contracts, two execution modes — the
  rust-analyzer/rustc precedent. The salsa tier carries explicit memory bounds
  (interning + LRU) so it never reproduces the "language server ate my RAM"
  failure mode. Its firewall is the **per-SFC summary** (exported props /
  emits / slots types, component references): the only cross-file salsa
  dependency, so a template-body edit never leaves the file unless the summary
  durably changes. Summaries follow the GHC `.hi` rule: **fingerprinted per
  declaration**, with consumers recording which declarations they used —
  invalidation is "any used fingerprint changed". App-global facts (global
  components, app-level provide/inject, dialect-wide directives) are the
  orphan-instance equivalent and live in a dedicated global summary with its
  own fingerprint. S3 code-shape decisions never enter a summary — contracts,
  not chosen optimizations. Incrementalization is hybrid — only genuinely recursive
  fact groups (graph reachability, route typing, transitive slots) are
  incremental; block-local facts recompute from content-keyed artifacts.
  _Below_ salsa sits a Lean-style **snapshot tree**: stage tasks at natural
  joints (header → block → S2 region) with one reuse rule — old syntax ≡ new
  syntax ⇒ adopt the old subtree — plus cascade-cancellation tokens, covering
  most keystroke traffic without pushing salsa finer.

## Extension contracts (decision 1)

Four published extension surfaces — three WIT contracts plus the JS plugin
tier; in-tree implementations are Vue-family only:

| Contract                                    | Plugs in at                                             | In-tree                                | External (examples)                         |
| ------------------------------------------- | ------------------------------------------------------- | -------------------------------------- | ------------------------------------------- |
| Input dialect                               | S1 parser + S1→S2 lowering                              | Vue 3, Vue 2 (`legacy`), SFC, JSX, pug | Svelte, Solid, Astro                        |
| Expression dialect                          | S2 `ExprRef` capability set                             | JS/TS (oxc)                            | MoonBit, Elixir-hosted                      |
| Output target                               | S3/S2 → S4 emitter                                      | VDOM, Vapor, SSR, virtual TS, `.d.ts`  | Volt (Elixir), other hosts                  |
| **JS plugins / custom rules** (charter #29) | S2 neutral-core view + fact query API, via napi/vitrine | rule authoring SDK                     | user-land lint rules, project-local plugins |

The JS tier serves end users (Vue developers write JS, not Rust) across all
four charter-#29 hook families, each with a defined boundary: **custom rules
and fact providers** see the neutral-core S2 view and declare fact demands
exactly like Rust rules, executing outside the fused walks in batched passes;
**compile transform hooks** join the pipeline at the single pre-canonical S2
point (per-block batches — compilation waits there and only there, and a
cache hit skips the join); **formatter/output hooks** attach after S4/format
emission, batched per document. Node-visit batches cross the napi boundary in
bulk, never per-node chatter.
Each plugin's cost is attributed in output (a slow rule is visible), and JS
rule results are content-key cached so user rules participate in
incrementality. The fused compile path and the Rust rule corpus never wait on
user JS.

Contract stability follows the `vize_marquette` precedent: versioned,
deterministic serialization at the boundary, compatibility classified as
additive vs breaking. **Decided linking model — two tiers:** first-party
dialects (Vue family, pug) are compiled in behind Rust traits and cargo
features (the `legacy` pattern: zero cost when off, no dynamic dispatch);
external dialects attach out-of-process over the serialized contract, which
sidesteps Rust ABI instability and keeps "in-tree is Vue-only" honest. The
external-tier transport is the **WASM component model (WIT interfaces)** —
typed, versioned, and thanks to the `wasm32-wasip2` core target the same
contract hosts a dialect out-of-process or in-process under wasmtime.
Practitioner reports suggest substantial throughput advantages over JSON-RPC;
our own transport benchmark (plan task P6-1) is the number that will actually
gate the choice. Interfaces are coarse-grained (whole block in, surface tree
out): the canonical ABI copies at every boundary, so per-node chatter is
banned by design. Two Swift-macro lessons apply: the WIT world carries a
capability handshake (protocol version + feature strings) from day one with a
**written compatibility policy**, and the extension SDK ships as a prebuilt,
versioned artifact — Swift's macro build-time crisis came from making every
plugin recompile the syntax library, not from the process boundary.
