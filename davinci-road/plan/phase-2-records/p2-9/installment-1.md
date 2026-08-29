# P2-9 Installment 1 — substrate + `v-if` (2026-08-21)

> [!NOTE]
> Part of the [P2-9 series record](../p2-9.md), split per installment
> under the 350-line source budget.

### The substrate home (the dependency-direction decision)

The contract says the S2 path lives "inside `vize_atelier_core`'s DOM
transform lane" — and the phase-2.md constraint says that crate, being
published, can never carry the Davinci crates in its release graph
(`tests/tooling/moonbit-publish-crates.test.ts`). The resolution splits
the substrate along exactly that line:

- **Pass bodies live in `vize_s1_to_s2::pass`** (`src/pass.rs` +
  `src/pass/vif.rs`). Ricalco is `publish = false`, already depends
  downward on `vize_davinci` (the P2-2 pass manager) and `vize_s2`
  (the ops), and the passes are the continuation of the dialect
  lowering — the MLIR conversion-library shape extended one step:
  `lower` converts, the passes legalize. Not in `vize_s2` (the
  neutral pivot must not learn Vue; P2-5a scoped it to the op family
  and verifier), not a new crate (premature for one pass; the series
  can split one out if the module outgrows the budget).
- **The dual-run comparator lives in `vize_atelier_core` test space**
  (`tests/s2_support/` + the two binaries), with `vize_davinci` /
  `vize_s2` / `vize_s1_to_s2` / `vize_sinopia` as
  **dev-dependencies**. The gate's rule, read from its source: a
  workspace dependency escapes the publish-order check iff
  `kind === "dev" && req === "*"` — a path-only dev-dependency cargo
  strips on publish. The contract's "dev-dependencies + optional
  feature deps" sketch was **narrowed to dev-deps only**: an optional
  feature dependency is `kind: null` in the metadata, would enter the
  published manifest, and the gate rejects it — verified by running the
  suite, all six tests green with the new edges in place.
- The in-`src` `#[cfg(any(test, feature))]` comparator shape (P1-7) was
  deliberately **not** used: it presumes a migrated read inside the
  shipped path to arm, and this installment migrates nothing shipped —
  the S2 lane runs _beside_ the legacy lane. When P2-11 puts a real S2
  read into a published crate, the program decision phase-2.md names
  (publish the Davinci crates / fold / feature-gate) must be made; this
  installment neither makes nor prejudices it.

### The lane flag (charter #26)

`VIZE_DAVINCI_TRANSFORM` — named once in `vize_s1_to_s2::pass`
(`TRANSFORM_LANE_FLAG`; ricalco is `no_std` and reads no environment),
read in the comparator: value `legacy` disarms the S2 dual-run,
counted (`skipped_legacy_flag`), never silent. The legacy lane is the
only shipped lane while the phase is live; the differential lane, not
the flag, carries the risk (phase-2.md § 11). The exit gate's deletion
grep has one home.

### The `v-if` port and its classification (the review point)

Three quarters of the old transform is already absorbed: chain grouping
(the enter/exit sibling mutation) happened at P2-8 lowering — a
region-owning `ui.if` is built with its branches from birth — and the
grammar diagnostics (missing expression, orphan else) are the
lowering's, with relief's exact text. Runtime-helper registration is
DOM-backend business and moves with P2-11. What remains, and is the
pass body: the branch-key half — `extract_key_prop`'s lift of the
authored static `key` into a per-branch fact, and the duplicate-key
diagnostic (vuejs/core #13881, wording pinned byte-identical to
`ErrorCode::VIfSameKey.message()` by the witness suite, since ricalco
must not depend on the legacy AST crate).

**`MandatoryLowering`, barrier**, pinned by test:

- _Mandatory_: skipping changes meaning — the key fact is what keyed
  branch reuse compiles from, and the duplicate-key error must fire at
  every optimization tier (the old transform ran unconditionally).
- _Lowering, not Diagnostic_: the pass **mutates** — the key leaves the
  element's syntactic surface and becomes a semantic fact — and it is
  the pass that establishes the canonical `ui.if` form; the verifier's
  rigor escalates to `Canonical` exactly when it completes (pinned:
  `the_vif_pass_is_the_canonicalizing_pass`). A diagnostic pass must
  preserve; this one does not.
- _Barrier_: forced by law 1, and independently true — the collision
  check reads facts **across** branches, not the single-visit locality
  `Fusable` claims.
- `Preserved::ALL`: attribute-to-fact movement invalidates no analysis.

Registered with the P2-2 pass manager: `TRANSFORM` is a const
`Pipeline` (`s2(v-if)`), grouping const-pinned (one group, fully
serialized), driven through `run_pipeline` by
`pass::run_transform` with the caller's observer plus the P2-6
`VerifyObserver` between passes in debug (`note` → `check` →
`check_table` per side table, per its module docs). Facts ride a
`SideTable<IfFacts>` keyed by the `ui.if` op's page-order id; the pass
re-derives ids by walking in print order and **asserts its count
against the lowering's minted accounting on every run**. Every
extraction and every diagnostic leaves provenance
(`pass.v-if.branch-key` / `error.v-if-same-key`).

Two extraction guards, both matching the legacy transform exactly:

- a `ui.for` branch root keeps its key with the iteration (Vue 3
  precedence);
- **the span-identity guard**: the single root op must _be_ the branch
  carrier (op span == branch span). Without it, an unwrapped
  `<template v-if><div key>` would steal the inner element's own key —
  a real over-extraction caught during design, pinned by
  `an_unwrapped_single_child_keeps_its_own_key`.

### TS-17

`crates/vize_s1_to_s2/tests/vif_pass_snapshot.rs`, the P2-4 harness
shape: two committed fixtures (`tests/fixtures/vif/chain-keys.vue`,
`collision.vue`) → lower → pipeline under the budget observer → full
normalized folio snapshot (`assert_folio_snapshot!`), with the walk
accounting (`walks=1 passes=1`), facts, and diagnostics as the
structural supplements. The snapshots show the moved surface: branch
keys gone from `attr` lines, the iterated `img`'s key still present.

### The differential lane (TS-25)

"Compared at the DOM-output level" is realized as the
**DOM-output-determining projection** of the if structure — chain order,
branch count/order, condition source text (trimmed), branch keys —
because no DOM backend exists to emit bytes from S2 until P2-11; TS-11
holds the actual output bytes still. Skips are counted classes, never
silence (module docs of `tests/s2_support/mod.rs` carry the full list).

- **Battery**: 18 committed templates, dual-run with **exact-pinned**
  counters in the plain witness (`davinci_s2_transform.rs`) and again
  in the corpus entry: 18 compared, 19 if-ops, 36 branches, 13 static
  keys, 2 dynamic, 1 template-wrapper key, 1 slot-outlet key, 0 skips.
- **Corpus** (read-only against the main checkout's hydrated fixtures,
  the P2-5b/P2-8 pattern; run twice 2026-08-21, identical — the
  P0-2/P0-5 convention):

  ```sh
  VIZE_DAVINCI_DIFFERENTIAL_CORPUS=tests/_fixtures/_git \
    cargo test -p vize_atelier_core --features davinci-differential \
    --test davinci_s2_transform_corpus -- --nocapture
  ```

  12,215 files, 194 without a template block, 12,021 templates seen,
  **12,017 compared, zero divergence** — 8,515 if-ops, 11,511 branches,
  45 static-key and 81 dynamic-key classifications. 4 skips, each a
  hard legacy parse error, named: misskey `MkNoteMediaGrid.vue` and
  `MkUrlPreview.vue` (MissingEndTag), nuxt-ui docs
  `ComponentCode.vue` (malformed markup), vue2-elm `shoplist.vue`
  (DuplicateAttribute). `skipped_s2_errors` = 0.

- **A measured domain decision**: the first cut skipped on _any_ legacy
  parse error — 3,027 of 12,021 templates, probed to be 7,304
  `ExtendPoint` **recovery notes** (spec repairs the parser already
  applied: self-closing rewrites and kin) plus a handful of hard
  errors. Recovery-note templates were then compared rather than
  skipped — zero divergence there too — so the skip predicate is
  `!ErrorCode::is_recovery` and the claim covers 99.97% of templates
  rather than 74.8%. Honest label: the corpus tree's hydration state is
  the main checkout's of this day (12 submodules carry modified
  content), so these are totality/agreement proofs over real files, not
  a pinned-count baseline.

### The 11.73% measurement seed

The P2-5b command, run twice from this worktree (identical both runs):

```sh
VIZE_DAVINCI_DIFFERENTIAL_CORPUS=/path/to/vize/tests/_fixtures/_git \
  cargo test -p vize_atelier_sfc --features davinci-differential \
  --test davinci_differential -- --nocapture
```

admitted 196,236; legacy total 28,636 of 224,872 calls (**12.73%**;
the committed P1-9 run's 11.73% is the same class on a different
hydration state) — `unretained` 21,876, `params` 4,614,
`dialect_rejected` 1,874, `ts_strip_rewrote` 272. **Byte-identical to
P2-5b's runs, as expected**: this installment's S2 lane runs beside the
shipped path and touches no `rewrite_expression` site, so the number is
the series' before-baseline, not a movement claim. The class can only
move when S2 region structure starts feeding the rewriter (later
installments / P2-5b's widening), and this seed is what that will be
measured against.

### TS-11 (deferred precisely)

No shipped compile path changed: the pass and comparator live in a
`publish = false` crate and dev-dep test space; `cargo tree -i` on the
Davinci crates still names only unpublished experimental crates, so no
output byte can move — the same mechanical argument every additive
Davinci task has used, and the six-test publish gate run green is its
enforcement. The worktree materializes no fixture submodules and
`corpus-diff.mjs` re-materializes `node_modules` (not runnable
read-only), so the all-clean sweep is deferred with the standing
recipe: `node tools/davinci/corpus-diff.mjs --surface compiler
--shards 2 --timeout-ms 600000` from clean fixtures.

### Other acceptance, clause by clause

- **TS-1**: `cargo test -p vize_s1_to_s2` (37 tests: 14 unit + the P2-8
  suites + 9 `vif_pass` + 2 snapshots + battery/elements/facts/shapes)
  and `cargo test -p vize_atelier_core` fully green; disegno / davinci
  / sinopia suites re-run green; the ricalco lowering corpus lane
  reproduces P2-8's exact census (12,215 / 10,416).
- **TS-13**: `assertion-lint: OK`, allowlist untouched — three
  substring assertions in the first cut were rewritten as exact
  structural oracles over the owned folio model rather than
  allowlisted.
- **House rules**: every new file ≤ 350 lines (largest:
  `pass/vif.rs` 334); no `mod.rs` under `src/`; ricalco stays
  `no_std + alloc` (wasm32-wasip2 build green via the P2-4 sysroot
  overlay); new fact types carry `'static` assertions and guarded size
  asserts (`BranchKey` 32 — the first guess of 40 was corrected by the
  assert failing, the ratchet working); clippy house invocation
  (`--workspace -- -D warnings -D clippy::wildcard_imports`) and
  `cargo fmt --check` clean, and every new target is additionally
  clippy-clean under `--tests`.
- **Benches**: none touched, none added — the TS-10 alloc re-record
  clause binds the installments that move hot paths; this one adds no
  code to any shipped path.
- **Registry**: TS-25's instance list gains the P2-9 transform lane
  (the phase-2.md registry-maintenance note).

### Gaps and owners (what later installments inherit)

- **Template-wrapper attributes**: a `<template v-if key="t">` key is
  dropped at lowering (`drop.template-attribute`) before any pass can
  see it — counted (`keys_template_if`) and pinned by test. The
  wrapper-attribute story (Vue renders that key on the branch fragment)
  belongs to the installment that gives wrapper facts a home.
- **Dynamic keys**: `:key` is still `defer.v-bind`; extraction and the
  dynamic arm of the collision check land with `ui.bind`
  (`transform_element`/`v_bind` installment). Corpus share measured:
  81 dynamic vs 45 static branch keys.
- **Pass catalogue for `davinci-opt`**: the binary still binds unknown
  pipeline names to no-ops; it cannot run ricalco's bodies because
  `vize_davinci` depending on `vize_s1_to_s2` would be a cycle. The
  name→body catalogue needs a home above both (P2-11's program
  decision is the natural point).
- **Comparator normalizations to revisit**: condition text compares
  trimmed (both lanes are trim-insensitive); entity-bearing conditions
  never diverged on this corpus but the S1 v1 no-decoding scope means
  they could — the class would be counted, not averaged, when it
  appears.
- The `v_for` port is the natural installment 2: `ui.for` and its
  region already exist from lowering, the splitter unification question
  from the P2-8 record is still open, and the comparator's projection
  extends to for-bindings the same way it did to branches.
