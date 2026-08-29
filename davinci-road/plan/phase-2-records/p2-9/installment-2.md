# P2-9 Installment 2 — `v-for` (2026-08-21)

> [!NOTE]
> Part of the [P2-9 series record](../p2-9.md), split per installment
> under the 350-line source budget.

### The absorbed-vs-pass-body split (the headline measurement)

Installment 1 found the v-if transform three-quarters absorbed; the
v-for measurement is stronger — the old step file
(`crates/vize_atelier_core/src/steps/v_for.rs`) ports **whole**
into the P2-8 lowering, and so do most of its driver halves:

- **the value split** (separator find, strict alias split) —
  `lower/vfor.rs`, the P2-5b decision consumed (`a in b in c` reads
  alias `a`, source `b in c`; the undecomposable whole is
  `Opaque(ForValue)`, pessimal);
- **both grammar diagnostics** (`VForNoExpression`,
  `VForMalformedExpression`) — the lowering's, relief's exact text;
- **the node restructure** (`transform_v_for`'s take-and-wrap) — a
  region-owning `ui.for` from birth, `<template v-for>` unwrap
  included;
- **scope recording** (`enter_v_for_scope`'s registration data in
  `lane/traverse.rs`) — P2-8's `ScopeFacts`, one fresh tag per
  site;
- **moved elsewhere, not here**: runtime helpers, keyed/unkeyed
  fragment decisions, the iterated element's `key` prop and `v-memo`'s
  placeholder params (all DOM realization — P2-11); source identifier
  prefixing (`process_expression` — the old lane's
  `transform_expression`, P2-5b's contract, the task's standing
  non-goal).

What remains — the pass body (`vize_s1_to_s2::pass::vfor`) — is the
**hygiene consumption** the series contract names: the pass is where
the lowering's recorded scope facts become load-bearing. Per `ui.for`,
in one page-order walk: entry present, tag fresh across the artifact,
bindings re-derived from the binding surface through the **same one
scanner** (`simple_identifier`, re-exported from the lowering — the
#4365 discipline) and asserted byte-equal with `Authored` origins at
the positions' exact spans; then the consumed view published as
`ForFacts { tag, value, key, index }` with each position `Named(name)`
/ `Pending` / `Absent`, under a `pass.v-for.scope` provenance record.
A broken law panics as a compiler bug (the id-accounting style), never
a user diagnostic. **The pass synthesizes no names** — a placeholder
for an absent alias is realization (P2-11), and the first
`ScopeOrigin::Synthesized` producer stays slot normalization, exactly
as the P2-8 record assigned it; pinned by
`an_absent_alias_consumes_as_absent_never_synthesized`.

One pessimal distinction was designed in deliberately: a zero-width
value hole under a **cleanly split** source is `Absent` (the valid
`v-for=" in xs"` binds nothing), while the same hole under a
`ForValue`-escaped source is `Pending` — the alias of an
undecomposable value is unknowable, and claiming absence would violate
opaque law pessimality. Pinned by
`an_undecomposable_value_is_pessimally_pending`.

### Classification (the review point)

**`MandatoryLowering`, barrier**, `Preserved::ALL`:

- _Mandatory_: the old lane entered the v-for scope unconditionally at
  every tier because binding resolution inside the region depends on
  it; a tier skipping this pass hands later stages unvalidated facts
  and no consumed view — meaning, not polish.
- _Lowering, not Diagnostic_: the pass emits **no user diagnostic**
  (both v-for errors are the lowering's), so `MandatoryDiagnostic` is
  simply false of it; what it does is establish an invariant later
  stages assume — `PassKind::MandatoryLowering`'s literal definition.
  **The taxonomy tension is recorded, not smoothed**: unlike v-if this
  is a _preserving_ mandatory pass (`the_pass_neither_mutates_nor_diagnoses`
  pins folio and diagnostics byte-identical across it); a "mandatory
  analysis" would be a fourth kind the P2-2 taxonomy does not offer,
  and the const pins keep the choice loud for review.
- _Barrier_: law 1 forces it, and independently tag freshness is a
  fact across every introduction site, not single-visit locality.
- **The fusion answer**: v-for fuses with nothing — mandatory bars it,
  and its only neighbour (v-if) is itself a barrier. Const-pinned:
  `TRANSFORM.group_count() == 2`, `is_fully_serialized()`, and the
  budget-observer pins moved `walks=1` → `walks=2` in both suites —
  the serialization's measured cost, re-pinned deliberately.

The page-order re-derivation (mint/skip arithmetic) was extracted to
`pass/walk.rs`, shared by both passes: the numbering law keeps exactly
one home, the P1-8 scanner-split lesson applied before a third copy
could exist.

### TS-17

`crates/vize_s1_to_s2/tests/vfor_pass_snapshot.rs`, two committed
fixtures → lower → pipeline → full normalized folio snapshots
(`assert_folio_snapshot!`): `tests/fixtures/vfor/loops.vue` (a keyed
`li` loop, a destructuring `<template v-for>` holding a keyed v-if
chain, an absent-alias loop) and `holes.vue` (the undecomposable value,
the expressionless `v-for`). The snapshots show the preserved surface:
the iterated `li` **keeps** `key="row"` (never lifted — legacy codegen
reads it per vnode), while the `dt` branch key inside the template loop
is gone (the vif pass's move — the two passes composing), and the
escapes print as `opaque(for-value …)`. Supplements pin `walks=2`,
the three consumed facts with tags `#0..#2`, and the exact two
lowering errors on `holes.vue`.

### The differential lane extension

The for projection is `renderList`'s whole input surface — source text
plus the three alias texts, trimmed, in document order. The iterated
element's `key` prop is **compared by neither lane in the for
projection, deliberately**: legacy never lifts it (element surface,
read per vnode at codegen), S2 leaves it on the element — it is
already covered by the element surface both lanes carry. Skip classes
gained: `for_compound` (a compound source/alias rebuild — never seen)
and the counted `for_values_absent` agreement; a malformed or
expressionless `v-for` skips as `skipped_s2_errors` pre-pass, matching
the legacy transform's refusal to build a `ForNode` from it (corpus
share: zero).

- **Battery**: 18 → 30 templates (the v-for half: grammar variants,
  destructure, no-parens pair, template wrapper, keyed element, absent
  alias, the `a in b in c` first-viable pin, nesting, slot iteration,
  the v-if/v-for precedence template). Exact-pinned in the plain
  witness and the corpus entry: 30 compared, if half 20 ops / 38
  branches (keys unchanged: 13/2/1/1), for half **15 ops, 14 values,
  4 keys, 1 index, 1 absent, 0 compound** — predicted before the
  first run and confirmed unchanged by it.
- **Corpus** (read-only against the main checkout's hydrated fixtures,
  the standing command; run twice 2026-08-21, byte-identical):
  12,215 files, 12,021 templates, **12,017 compared, zero
  divergence** — the if half exactly installment 1's numbers, plus
  **2,882 for-ops: 2,882 value, 1,083 key, 3 index comparisons; 0
  absent-alias, 0 compound**. The 4 skips are the same named
  hard-parse-error files; `skipped_s2_errors` = 0 (no real template
  carries a malformed v-for). The entity-bearing class again never
  appeared — now measured over for sources and aliases too, same
  count-not-average rule standing if it ever does.

### The P2-8 splitter-unification question, answered by measurement

P2-8 left open whether its Vue-grammar splitter and the shipped one
should unify "after P2-9 measures what the transform port needs". The
measurement: **the pass needed zero grammar** — the split is entirely
pre-pass (lowering) on the S2 side and entirely `parse_for_expression`
(transform) on the legacy side, and the corpus lane now proves the two
agree on every real template (2,882 for-ops, zero divergence, the
strongest agreement proof the duplication will ever get). The ~120
duplicated lines stay confined to their two homes; unification before
the exit gate would invert the strangler (the new lane depending on
the legacy crate), and at the exit gate the legacy copy is deleted
instead.

### The residual class: measured, unmoved

The P2-5b command, run twice from this worktree (byte-identical):
admitted 196,236; legacy total 28,636 of 224,872 = **12.73%**
(`unretained` 21,876, `params` 4,614, `dialect_rejected` 1,874,
`ts_strip_rewrote` 272). The contract flagged this installment as the
first that might move the number (the reparse class contains v-for
values); measured: **it does not move**, because the pass feeds no
`rewrite_expression` site — nothing S2 produces is consumed by the
shipped prefixer yet. What this installment _does_ contribute is the
prepared feed: `ForFacts`/`ScopeFacts` are exactly the region-scope
data the P2-5b widening will read when S2 structure starts driving the
rewriter; the movement claim stays with that future installment.

### Other acceptance, clause by clause

- **TS-1**: `cargo test -p vize_s1_to_s2` — 63 tests (16 unit + the
  P2-8 suites + 9 `vif_pass` + 2 vif snapshots + 9 `vfor_pass` + 2
  vfor snapshots) green; `cargo test -p vize_atelier_core` fully
  green; disegno / davinci / sinopia suites green; the ricalco
  lowering corpus lane reproduces P2-8's exact census
  (12,215 / 10,416).
- **TS-13**: `assertion-lint: OK`, allowlist untouched; every new pin
  is whole-value exact equality (fact structs, provenance triples,
  attribute lists, folio text).
- **TS-11, deferred precisely**: no shipped path touched — no
  `Cargo.toml` changed, so the dependency graph the publish gate
  checks is bit-identical to installment 1's green run and the
  mechanical argument stands unchanged; the clean-fixture
  `corpus-diff` sweep recipe stands as recorded there.
- **House rules**: every file ≤ 350 (largest new: `pass/vfor.rs` 317);
  no `mod.rs` under `src/`; ricalco stays `no_std + alloc`
  (wasm32-wasip2 build green via the P2-4 sysroot overlay); `ForName`
  24 / `ForFacts` 80 size asserts (first guesses held this time);
  clippy house invocation clean, the new/touched test targets
  additionally clippy-clean under `--tests`; `cargo fmt --check`
  clean.
- **Benches**: none touched, none added — no shipped path gains code.

### Gaps and owners (what later installments inherit)

- **Template-wrapper attributes, now covering v-for too**: a
  `<template v-for="…" key="…">` key is dropped at lowering
  (`drop.template-attribute`) exactly like the v-if wrapper key —
  invisible to the for projection by design (keys are element
  surface). One wrapper-facts home should land for both when an
  installment needs it; until then both are recorded drops.
- **Dynamic `:key` on iterated elements**: still `defer.v-bind`;
  extraction lands with `ui.bind` (unchanged from installment 1).
- **The middle-hole param note for P2-11**: on `(item, , i)` both
  lanes record key `None` + index `i` and agree, but legacy codegen
  prints `(item, i)` where upstream synthesizes a `__` placeholder
  (`createParamsList`) — a legacy-vs-upstream question outside this
  lane's domain (corpus share: at most the 3 index-bearing templates),
  recorded so P2-11's DOM backend decides it consciously, with the
  hygiene machinery ready for the synthesized name it may need.
- **Next installment**: `v_slot` normalization is the natural 3 — it
  owns the first `ScopeOrigin::Synthesized` producer (P2-8's
  assignment), the scope-consumption pattern this installment proved
  extends to slot props directly, and the `defer.v-slot` classes are
  its checklist. `transform_text` is the alternative when a smaller
  step is wanted (it unlocks the `Compound` opaque producer).
