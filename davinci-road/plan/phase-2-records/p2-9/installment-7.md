# P2-9 Installment 7 — the legacy dialect behind `_legacy` (2026-08-21)

> [!NOTE]
> Part of the [P2-9 series record](../p2-9.md), split per installment
> under the 350-line source budget. This is the closing installment.

### The absorbed-vs-pass-body split (the headline measurement)

Unlike installments 4–5's dead step files, `legacy.rs` (306 lines) and
`legacy_filters.rs` (337) are **both live in the shipped lane** — and
feature-gated, the only step files that are: `lane.rs:276` runs
`desugar_legacy_template` pre-traversal (`.sync` expansion,
`slot-scope`/`scope` conversion, gated on the one `scoped_slot_attrs`
capability), `transform/element.rs:161` runs `desugar_v2_v_on_modifiers`
per directive (V2 only), and `process_expression` consults the filter
splitter. Three measurements shaped the port:

- **The V2 dialect is parse-identical to V3.** Every parse-lane
  capability in `vize_armature::legacy` (triple-mustache raw HTML,
  attr-value interpolation, `v-repeat`, clause-style arguments) is a
  0.x/1.x surface; V2's whole legacy story is transform-lane sugar over
  a V3-compatible parse. Sinopia's S1 tree is therefore already correct
  for V2 sources and the S2 port needs no S1 change; the 0.x/1.x lines
  stay out of S2 scope (no S1 parse story — recorded, with the V1
  filter capability unit-pinned since filters span every line).
- **The shipped filter rewrite is prefixing-coupled.** Every
  `process_expression` call site is `prefix_identifiers || is_ts`-gated
  (interpolations, directive values, conditions, v-for sources, keys),
  so under default options the live lane never splits a chain — and
  under prefixing it splits _every_ expression position, Vue 2's
  documented mustache + `v-bind` scope included but not limited to it.
  The S2 split keys on the **dialect alone** at the documented
  positions; the differential lane holds both deviations as counted
  classes (below).
- **The codegen half is realization.** `ctx.add_filter` →
  `RootNode::filters` (relief's cfg-gated field, the 224→248 size-assert
  pair) → `codegen/root.rs`'s `_resolveFilter` preamble is the emitting
  half; only the transform-lane state (the registration order itself)
  ports, as the legacy pass's fact.

### The dialect-op test (the record's decision, and the flagship)

The contract says "legacy behaviors as `vue.*` dialect ops"; the
measured rule that decided which ones: **does the surface have an exact
modern equivalent the shipped lane itself rewrites to?**

- `.sync`, `slot-scope`/`scope`, `.native` + numeric keycodes: **yes** —
  the shipped desugars are the living code, and the port follows it:
  the same rewrites mirrored byte-for-byte at lowering into the ops the
  family already speaks (`ui.bind` + the appended `ui.on
"update:<name>"` with the exact `$event => ((exp) = $event)` handler
  and the same-name-shorthand camelized value; `ui.slot-content` with
  the companion `slot` name consumed; modifier strip/rename), each
  under a `normalize.legacy.*` / `consume.legacy.*` provenance record
  so nothing launders silently. The fairness rule is satisfied the
  installment-5 way (the parser-shorthand precedent): the products are
  dialect-neutral vocabulary, the spelling is recorded.
- **Filters: no** — a chain has no modern form, and under the dialect
  `a | f` is not a JS expression at all (`|` is the filter separator —
  the `ForValue` wrong-grammar argument, dialect-shaped). The flagship
  lands: a lone filter interpolation lowers to the **`vue.filter`**
  region op, a filter-bearing `v-bind` value to the new
  **`OpaqueReason::LegacyFilter`** escape, both pessimal from the first
  byte, both with the split recorded beside the tree
  (`Lowered::filters`, the Compound producer's pattern) — the splitter
  mirrored byte-for-byte from the shipped `parse_filters`
  (`lower/legacy/filters.rs`, the installment-2 duplication decision:
  two homes, differential-proven agreement, exit-gate deletion).

### Zero cost when off: the mechanism and its pin

Everything is cfg-gated — the op variant and opaque reason included
(`vize_disegno/_legacy`), the lowering hooks, the pass, the pipeline
(`vize_ricalco/_legacy`) — so the off-build compiles **none** of it.
The canary-soundness decision the contract asked for, recorded:

- The repo has **no** feature-gated enum variant precedent (measured:
  relief gates fields, atelier gates modules), so the variants got the
  ritual instead: injected first, the canary captured broken (`E0004`
  on `op_keyword` and `reason_keyword`, `_legacy` shape), then fixed
  with cfg'd arms. **The pin is the canary's own two-sidedness**: in
  the off shape an unguarded `vue.*` arm is itself a compile error
  (the variant does not exist in the type — the strongest possible
  compiles-out witness), in the on shape a missing arm is `E0004`.
- The unification rule that keeps cfg'd variants sound:
  `vize_disegno/_legacy` is enabled **only** through
  `vize_ricalco/_legacy`, and `vize_atelier_core`'s dev-dependency pins
  `_legacy` unconditionally, so every graph that materializes the
  variants also compiles their arms (the comparator's arms are
  unconditional for exactly this reason).
- Behavioural pins, both directions: the plain `TRANSFORM` pipeline is
  byte-identical in both shapes (the legacy pass rides only the
  per-dialect `TRANSFORM_LEGACY`; every `walks=6` pin holds unchanged),
  a filter-free template lowers byte-identically through both entry
  points, and the **corpus ran twice per feature shape with
  byte-identical counters across all four runs** — the feature ON moved
  nothing anywhere.
- Mechanically exercised per shape: build, full test suite, clippy
  (house + `--tests`), and wasm32-wasip2 (disegno + ricalco, the P2-4
  sysroot overlay) — all green with and without the feature.

### The dependency-direction lesson (caught by the acceptance run)

The first cut forwarded `vize_ricalco/_legacy` →
`vize_armature/legacy`, which forwards to `vize_relief/_legacy` — whose
cfg-gated AST fields (`InterpolationNode::raw`) break any
workspace-unified build of a crate that constructs relief nodes without
its own mirror feature: `cargo check --workspace --tests` failed in
`vize_atelier_jsx` (E0063), the exact hazard its manifest documents.
Resolution: ricalco carries a documented **capability mirror**
(`LegacyVueLine` + the three consumed fields) instead of the armature
edge — the installment-2 splitter precedent again — and the legacy
witness pins the mirror **field-for-field against the armature model**
where both homes are visible
(`the_ricalco_capability_mirror_matches_the_armature_model`). The
copies can only drift loudly; the exit gate deletes the legacy one.

### Classification (the review point)

**`pass::legacy` — `MandatoryLowering`, barrier, `Preserved::ALL`** —
the preserving-mandatory taxonomy tension's fourth occurrence,
const-pinned like its three predecessors. Mandatory: under a filter
dialect the asset registration is meaning at every tier (the live lane
registers unconditionally). Lowering, not Diagnostic: no user
diagnostic exists (the live rewrite bails silently on malformed names —
mirrored at the split), and the pass establishes the canonical
first-seen asset order realization resolves from. Barrier: law 1, and
independently the order is a cross-artifact fact. The pass consumes
every recorded split under three laws (entry ⟺ op, count-matched both
ways; re-splitting the opaque source through the one mirrored splitter
reproduces the parts byte-equally) and publishes `LegacyFacts`
(assets + per-site views).

**The pipeline is per-dialect** — the shape decision this installment
records: `TRANSFORM` stays six passes in both feature shapes;
`TRANSFORM_LEGACY` appends the legacy barrier as a seventh lone group
after the fusable singleton (`group_count() == 7`, const-pinned; the
hoist group stays a singleton — its new neighbour is a barrier). The
pass set is dialect-determined exactly as the shipped
`TransformOptions` arms `hoist_static`; legacy snapshots pin `walks=7`,
every plain suite keeps `walks=6`.

### The differential lane (the legacy battery, and the V3 control)

A new witness (`davinci_s2_transform_legacy.rs`, `--features legacy`)
dual-runs the 19-template committed legacy battery twice over:

- **Under the V2 dialect** — the shipped lane at default options (its
  desugars are dialect-gated, not prefixing-gated) against
  `lower_legacy(V2)` + `TRANSFORM_LEGACY`. Because the desugars are
  mirrored, every existing projection compares **directly** — the
  `.sync` product pair folds back into the authored contract on both
  sides by the same span-sharing rule the v-model reconstruction uses
  (the S2 fold is inert under V3: no two S2 binding ops ever share a
  span, pinned by the plain witness's unchanged counts). Exact-pinned:
  19 compared, zero divergence — 1 chain; slots 3 units / 3 groups /
  3 params / 1 invented; text 12 units (5 static + 8 dynamic parts,
  1 compound); surfaces 25 owners / 2 attrs / 3 + 1-dynamic binds /
  4 ons / **4 reconstructed `.sync` contracts**; and the legacy
  extras — **6 filter sites / 8 segments compared against the shipped
  `parse_filters` itself** (made `pub` under the feature, the
  installment-6 visibility precedent), **17 templates asset-matched
  against a filter-armed second legacy run** (V2 + prefixing, the
  hoist-armed-run pattern) with 2 counted narrowings (subset asserted
  as an order-preserving subsequence, never averaged), 1
  `filters_other_positions` probe (the v-if condition chain), 1
  `filters_in_compounds` probe (the merged run), 4 syncs / 2
  scoped-slots / 2 natives / 1 keycode mirrored. Every extra was
  predicted before the first run and confirmed by it; the text/surface
  tallies were taken from the first run after a per-class audit (the
  installment-4 honesty convention).
- **Under the default dialect** — the same 19 sources through the plain
  comparator, hoist half included: 19 compared, zero divergence
  (filters mean bitwise-or, `.sync` stays a modifier, `slot-scope`
  stays an attribute — surfaces 5 attrs / 7 binds / 0 models, hoist
  24 verdicts) — the shipped `legacy_filters.rs` suite's V3-inertness
  claim, now cross-lane.
- One parity finding pinned rather than smoothed: `slot-scope` on a
  **plain element** desugars to a v-slot the shipped validation then
  rejects (`VSlotMisplaced`) — Vue 2 allowed the spelling, the shipped
  implementation does not, and the S2 mirror reproduces the same
  diagnostic (`a_scoped_slot_on_a_plain_element_misplaces_exactly_like_the_live_lane`).

**Corpus** (read-only against the main checkout's hydrated fixtures,
the standing command; run **twice per feature shape**, 2026-08-21, all
four byte-identical): 12,215 files, 12,021 templates, **12,017
compared, zero divergence** — every counter equal to installment 6's
run to the digit, with `--features davinci-differential` and with
`--features davinci-differential,legacy` alike. The legacy code rides
the whole corpus and moves nothing: the zero-cost clause measured at
corpus scale.

### The residual class: measured, unmoved — the series' final word

The P2-5b command, run twice from this worktree (byte-identical):
admitted 196,236; legacy total 28,636 of 224,872 = **12.73%**
(`unretained` 21,876, `params` 4,614, `dialect_rejected` 1,874,
`ts_strip_rewrote` 272). Seven installments, one number, one structural
reason recorded once and cited since: the S2 lane feeds no shipped
`rewrite_expression` site, so no installment of this series could move
it. What the series hands P2-5b's widening and P2-11's read is the
prepared feed: `IfFacts`/`ForFacts`/`SlotFacts`/`TextFacts`/
`ModelFacts`/`StaticFacts`/`LegacyFacts` beside every op the old lane
still re-derives from scratch.

### TS-17

`crates/vize_ricalco/tests/legacy_pass_snapshot.rs`, two committed
fixtures → `lower_legacy(V2)` → the legacy pipeline → full normalized
folio snapshots: `tests/fixtures/legacy/filters.vue` (lone chains as
`vue.filter` lines, the bind chain as the opaque value on its
`ui.bind`, the logical-or and malformed-name bails as ordinary `js`,
the merged run still Compound) and `sugar.vue` (stripped `.sync` binds
with their appended `update:` listeners — camelized same-name value
included — the kept dynamic-argument `.sync`, rewritten v-on modifiers,
scoped-slot spellings as appended `ui.slot-content`, and the
conflict/plain-attr bails). Supplements pin `walks=7 passes=7`, the
asset order (`capitalize, f, g, formatId, quote, h` — dedup across
sites), the exact per-rule provenance counts, and empty diagnostics.

### Other acceptance, clause by clause

- **TS-1**: `cargo test -p vize_ricalco` green in both shapes (plain
  20 targets — byte-identical behaviour; `_legacy` 23 targets: +8
  `legacy_pass`, +11 `legacy_pass_sugar`, +2 snapshots, the mirror
  probe); `vize_disegno` green in both shapes (the canary grew its
  cfg'd arms; the `vue.filter` folio line round-trips —
  `the_legacy_filter_line_round_trips`); `cargo test -p
vize_atelier_core` fully green plain (22 targets) and with `legacy`
  (23); davinci / sinopia suites green. The lowering corpus lane re-run
  twice, identical: 12,215 files, 12,215 checked, with_diagnostics
  **804** — unchanged from installments 5–6.
- **Metamorphic (TS-21)**: matrix plane census unchanged (321
  mutations); the full corpus run twice with identical counts,
  **179,992 mutations, zero divergences**.
- **TS-13**: `assertion-lint: OK`, allowlist untouched — the one
  first-cut `contains` assertion was rewritten as the exact
  order-preserving subsequence oracle.
- **TS-11, deferred precisely**: no shipped compile path changed — the
  published-crate edits are the feature-gated visibility widening
  (`steps.rs`: `legacy_filters` `pub(crate)` → `pub`, inside the
  existing `legacy` cfg — the installment-6 precedent) and
  `vize_atelier_core`'s **dev-dependencies** (stripped on publish; the
  six-test publish gate re-run green after the manifest change, with
  the MoonBit registry symlinked per the standing recipe). The
  dependency graph the gate checks is unchanged for every default
  build; the clean-fixture `corpus-diff` sweep recipe stands as
  recorded in installment 1.
- **House rules**: every new file ≤ 350 after the deliberate splits
  (`lower/legacy.rs` 265 + `legacy/filters.rs` 221 + `legacy/sugar.rs`
  239; `pass/legacy.rs` 297 + `legacy/pipeline.rs` 110; the ricalco
  suite split `legacy_pass.rs` 186 + `legacy_pass_sugar.rs` 322; the
  comparator split `legacy.rs` 217 + `legacy_batt.rs` 85 +
  `legacy_filters_check.rs` 217); no `mod.rs` under `src/`; ricalco
  stays `no_std + alloc` with **no new dependency** (the armature edge
  deliberately not taken — above); wasm32-wasip2 green for disegno +
  ricalco in both shapes; size asserts `VueFilterOp` 24 /
  `FilterSegment` 48 / `FilterParts` 48 (first guesses held); clippy
  house invocation clean and the new/touched targets clippy-clean under
  `--tests` in both shapes (one first-cut finding: `manual_contains`);
  `cargo fmt --check` clean; the fuzz workspace `cargo check`-green.
- **Benches**: none touched, none added — no shipped path gains code.
- **TS-12**: `croquis-consumption.md` regenerated (the new test files
  move the naive consumer counts); `--check` green.
- **Registry (the phase-2.md maintenance note)**: TS-25's instance list
  already names the P2-9 transform lane (installment 1's addition,
  confirmed in `test-suites.md`); the paragraph's remaining debts are
  other tasks' (TS-22's column is P2-12b's, the P2-18 feed entry is
  P2-18's, TS-25's P2-11/P2-16 instances land with those tasks) —
  nothing in it is still owed by P2-9.

### Gaps and owners (the closing hand-off)

- **`filters_other_positions`** (battery 1, corpus 0 by construction —
  V3 corpus): the shipped rewrite's prefixing-coupled everywhere-scope
  vs the S2 dialect-scoped mustache + `v-bind` split. Owner: the
  DOM-legacy realization decides whether the extra positions are
  contract or accident; the armed-run subset law keeps the narrowing
  one-sided until then.
- **`filters_in_compounds`** (battery 1): a chain inside a merged run
  stays the Compound producer's part; the V2 re-admission of part
  texts is realization's, with the split rule one call away.
- **Outlet legacy sugar**: the desugars cover element and component
  owners; a `<slot slot-scope>` (or `.sync` on an outlet) is out of the
  mirrored scope — recorded, battery-free, realization-owned.
- **0.x/1.x lines**: no S1 parse story (their legacy surfaces are
  parse-lane); the S2 legacy story is V2-scoped with the V1
  filters-only capability pinned. `space_separated_filter_args` is
  ignored by the shipped splitter too — a mirrored quirk, not a gap.
- **The sync-fold modifier blind spot**: extra modifiers on a `.sync`
  bind vanish from both lanes' folded contract (the legacy fold's own
  shape, mirrored) — visible again at realization.
- **The hoist half stays V3-scoped** for the legacy battery (its
  oracles were measured on the default dialect); a legacy-dialect hoist
  differential re-runs the same lattice over desugared products and is
  deferred with the exit gate.
- **The capability mirror** and the **filter-splitter mirror** are the
  exit gate's to delete with the legacy lane itself, alongside the
  in-phase flag (`VIZE_DAVINCI_TRANSFORM`) — one grep, one home
  (`pass.rs`), P2-20's deletion list.
