# P2-9 Installment 4 — `transform_text` (2026-08-21)

> [!NOTE]
> Part of the [P2-9 series record](../p2-9.md), split per installment
> under the 350-line source budget.

### The absorbed-vs-pass-body split (the headline measurement)

The transform_text measurement is the series' strangest yet: the old
step file (`crates/vize_atelier_core/src/steps/text.rs`,
200 lines) is **dead code in the shipped lane** — exported from
`steps.rs`/`lib.rs`, called by nothing (measured: zero call sites
outside the two re-exports). The behaviour it describes ships from two
other places, and the port followed the living code, not the dead file:

- **Whitespace condensing** ships at **parse time**
  (`vize_armature/src/parser/whitespace.rs`, driven by the DOM
  configuration's `is_pre_tag`), before the legacy transform lane ever
  runs.
- **Text/interpolation merging** ships at **codegen time**
  (`codegen/children.rs`: maximal runs of consecutive
  text/interpolation children become one `createTextVNode` with a
  concatenated payload).
- **Both halves absorb into the P2-8 lowering**
  (`crates/vize_s1_to_s2/src/lower/text.rs` + `text/condense.rs`), for
  one decisive reason: **comments**. The remove-vs-condense rule reads
  comments as non-text-like neighbours, and run merging must break at
  them (`a<!--c-->b` is two text vnodes); comments exist only in S1 —
  the lowering drops them — so a post-lowering pass is comment-blind
  and wrong on real shapes (`a<!--c-->\n<!--d-->b` must lose its
  whitespace; a blind pass would keep a space). The P2-5b record
  independently requires the placement: the position-classified opaque
  reasons, `Compound` included, "are assignable only by the S1→S2
  lowering". Both computations are pinned comment-exact by test
  (`text_pass.rs`, the two comment cases).
- **The pass body** (`vize_s1_to_s2::pass::text`, a preserving pass —
  the vfor shape, third occurrence of the recorded taxonomy tension):
  the **compound consumption**. Per compound op it validates
  entry-present ⟺ compound-op (count-matched both directions), part
  shape (≥ 2 parts, ≥ 1 dynamic, no adjacent statics), **span tiling**
  (parts tile the op's span exactly — also the proof no comment or
  dropped byte was merged across), and the rebuilt source byte-equal
  through the one shared rebuild rule (`rebuild_source`, the #4365
  one-scanner discipline applied to spellings); then publishes
  [`TextFacts`]. It also holds the region adjacency law: text-family
  siblings are always separated by dropped source bytes — mergeable
  adjacency does not survive the lowering.
- **Stays realization (P2-11):** runtime-helper registration
  (`ToDisplayString`, `CreateText`), the single-space
  `createTextVNode()` convention, array forcing, and comment vnodes.

### Where condensing landed (the contract's RECORD decision)

**The lowering — not the pass, not realization.** Measured: the legacy
lane condenses at parse time (not transform time; the transform-lane
file that claims the behaviour is dead) and merges at codegen time.
The S2 analog of "parse time" is the S1→S2 conversion — the earliest
stage where semantics enter (S1 is the lossless record) and the last
that still sees comments. `<pre>` subtrees are exempt via the shipped
`is_pre_tag` (`tag == "pre"`,
`crates/vize_atelier_dom/src/compile/stage_options.rs`); rawtext
content (`script`/`style`/… — the metamorphic suite's nine-tag list)
is additionally exempt because a whole-SFC lowering must not collapse
whitespace inside another language (JS ASI). The comparator's legacy
parse now passes the shipped `is_pre_tag` too (the default `|_| false`
would condense inside `<pre>`, which no shipped compile does);
`is_pre_tag` feeds only the condense strategy, and every pre-series-4
projection held its exact counts under the change.

### The Compound producer (the series' first, and its pessimal frame)

A mixed run lowers to one `ui.interpolation` whose expression is
`Opaque(Compound)` — the class P2-5b reserved, produced nowhere until
now. The pessimal laws bind from the first byte: never constant, equal
to nothing, byte-verbatim-or-refusal — so the rebuilt
`OpaqueExpr::source` is a **display form** (canonical template
spelling: static parts verbatim, dynamics as `{{ <trimmed> }}`), and
nothing downstream may compile from it. What realization compiles from
is the recorded structure: the lowering writes each merged run's parts
(text, authored span, static/dynamic) into `Lowered.texts`, and the
pass publishes the validated view as `TextFacts` — the ForFacts
pattern: the opaque satisfies totality pessimally, the fact carries
the meaning. The retained-AST downgrade is deliberate and recorded: a
merged interpolation's `js(...)` becomes a part string (P1-5's
retention is per-position, and the merged position never existed as
one authored expression); realization re-admits part texts through the
one total rule when it needs ASTs.

### The metamorphic ratchet, and what its canary caught

P2-15 left open: "the P2-9 series should ratchet [`merge_text` and
`condense`] out (turn them off and watch the suite stay green)". Done
here — `normalize.rs` now declares **span elision only** for
wrap/split/merge/whitespace (attribute sorting stays, reorder-only),
and the deleted rules' sensitivity pins re-pin the same real
differences against the _lowering's_ canonical operations with no
normalization at all.

The ratchet immediately earned its keep: the first cut classified
whitespace **per node**, and the corpus canary caught two real bugs on
split-adjacency shapes (never emitted by the parser, always emitted by
the split mutator):

1. a split mixed text (`"\n    " + "Visit\n    "`) had its
   whitespace-only half stripped as a leading node, diverging from the
   one-node spelling (`" Visit "` vs `"Visit "`);
2. a split whitespace run's `Drop`-planned tail broke the merge scan,
   leaving `ui.text " "` + `ui.interpolation js(…)` where the one-node
   spelling merges a compound.

The fix is the **text-group** model (`lower/text/condense.rs`):
whitespace classification runs over maximal span-contiguous text
groups (one node each on parser output — the algorithm is then
armature's exactly), and the merge scan consumes a condensed run's
dropped tail, folding its bytes into the preceding part's authored
range so parts still tile. Both bug shapes are now green over the full
corpus with **zero declared normalization**, which is a strictly
stronger oracle than P2-15 shipped.

### Classification (the review point)

**`MandatoryLowering`, barrier**, `Preserved::ALL`:

- _Mandatory_: merged units are what DOM text emission compiles from —
  `createTextVNode` boundaries and payloads are meaning — and the
  legacy lane computed both halves unconditionally at every tier
  (parse and codegen have no optimization levels).
- _Lowering, not Diagnostic_: the pass emits no user diagnostic
  (neither half does in the legacy lane either); it establishes the
  invariant later stages assume — installment 2's preserving-mandatory
  tension, third occurrence, const-pinned.
- _Barrier_: law 1 forces it, and independently the adjacency law
  reads across sibling ops of every region.
- **The fusion answer**: four lone barriers, const-pinned
  (`group_count() == 4`, `is_fully_serialized()`); the walk pins moved
  `walks=3` → `walks=4` in the four suites that hold them (`vif_pass`
  - the three snapshot suites) — the serialization's measured cost,
    re-pinned deliberately. The pass drives its own shaped recursion
    over the shared `PageWalk` (flat visitation cannot hand a region its
    sibling pairs); the mint arithmetic keeps its one home in
    `pass/walk.rs`.

### The differential lane extension

The text projection is the merged unit surface: per template, the
document-order list of units (one per `createTextVNode` boundary),
each a sequence of static/dynamic parts compared by kind then text.
The legacy side re-groups its transformed tree exactly as its codegen
does (consecutive text/interpolation children, comments breaking
runs); the S2 side's remaining text-family ops _are_ the units — a
compound op contributes its recorded parts, so the comparison is also
the cross-lane witness for the Compound producer. Counted classes,
measured then decided:

- `entity_templates` — the legacy parser decodes entities in text and
  interpolation content; S1 v1 deliberately does not (the recorded
  deviation, now measured over text: **88 corpus templates**, skipped
  as a class, never averaged).
- `vpre_templates` — the legacy parser honours `v-pre` at parse
  (interpolations inside become text) and then erases the directive
  from its tree, while S2 defers it (`defer.v-pre`); the deterministic
  detector is the S2 lowering's own deferral record. Corpus share:
  **0** — the class exists in the battery only.
- `rawtext_excluded` — rawtext content excluded lane-neutrally by
  authored tag, S2-counted: **6 corpus ops**.
- `parts_compound` — a legacy compound interpolation content has no
  single text; **0** everywhere under default options.

- **Battery**: 44 → 58 templates (the text half: merge, lone nodes,
  both comment-boundary cases, condense variants, pre, rawtext,
  entity, v-pre, units inside if branches and for regions).
  Exact-pinned in the plain witness and the corpus entry. Honest
  label, unlike installments 2–3: the structural halves were predicted
  and confirmed (if 22/41, for 16/15, slots unchanged), but the text
  tallies' first hand count was wrong (76/66/15/5 predicted) and the
  pins were taken from the first run — **83 units, 70 static + 23
  dynamic parts, 7 compounds, 1/1/1/0 counted classes** — then
  confirmed unchanged by every later run.
- **Corpus** (read-only against the main checkout's hydrated fixtures,
  the standing command; run twice 2026-08-21, byte-identical, re-run
  twice after the canary fixes with identical counts): 12,215 files,
  12,021 templates, **12,017 compared, zero divergence** — the if, for
  and slot halves exactly installment 3's numbers (the `is_pre_tag`
  option change moved nothing), plus **37,575 text units, 28,783
  static and 11,588 dynamic parts compared, 2,135 compound units**;
  counted classes 88 entity-skipped / 0 v-pre / 6 rawtext-excluded /
  0 legacy-compound. The 4 skips are the same named hard-parse-error
  files; `skipped_s2_errors` = 0.

### The residual class: measured, unmoved

The P2-5b command, run twice from this worktree (byte-identical):
admitted 196,236; legacy total 28,636 of 224,872 = **12.73%**
(`unretained` 21,876, `params` 4,614, `dialect_rejected` 1,874,
`ts_strip_rewrote` 272). The contract flagged compound expressions as
a member of the reparse residual and this installment as a possible
first mover; measured: **unmoved**, for the standing reason — the S2
lane feeds no shipped `rewrite_expression` site, so nothing the merge
produces is consumed by the legacy prefixer. `TextFacts` joins
`ForFacts`/`SlotFacts` as the prepared feed the P2-5b widening will
read.

### The 5 comment-filler slots (the handed-forward class)

**The class stays open — and the port did not require a comment op.**
The installment-3 worry was that text handling would need comment
_visibility_; the lowering absorption dissolved it (S1 still has the
comments), so run boundaries and condense decisions are comment-exact
without any `vize_s2` addition. What remains missing is comment
_output_: the 5 corpus units are comment-only slot content — real DOM
in the legacy lane, invisible to S2 — and `units_filler_default`
re-measured at exactly 5. The class belongs to whatever installment
gives comments an S2 op; the op-family canary was therefore never
exercised here (no new op, deliberately).

### TS-17

`crates/vize_s1_to_s2/tests/text_pass_snapshot.rs`, two committed
fixtures → lower → pipeline → full normalized folio snapshots:
`tests/fixtures/text/merge.vue` (a five-part compound with an interior
run condensed, a comment-punched `<p>` keeping two units, indentation
runs removed) and `condense.vue` (the kept no-newline space between
elements, removed newline runs, interior collapse, and a `<pre>`
compound with its bytes verbatim). Supplements pin `walks=4`, the
recorded parts, the exact `pass.text.compound` /
`condense.drop-whitespace` / `condense.whitespace` provenance, and the
count-matched fact tables. The vif/vfor/vslot snapshots regenerated
under the condense (whitespace-only line removals, one id re-pin in
`vslot_pass.rs`) and were reviewed line by line — content ops
untouched.

### Other acceptance, clause by clause

- **TS-1**: `cargo test -p vize_s1_to_s2` — 95 tests (18 unit + the
  P2-8 suites + vif/vfor/vslot suites + 8 `text_pass` + 2 text
  snapshots + 7 metamorphic) green; `cargo test -p vize_atelier_core`
  fully green (witness suite 6 tests); disegno / davinci / sinopia
  suites green (untouched crates — no op, no verifier rule, no S1
  change). The ricalco lowering battery census moved and was re-pinned
  deliberately in both lanes: **(78, 33, 101, 1)** — whitespace-only
  text ops leave the artifact (89 → 78), their records become
  `condense.*` ones (107 → 101), and diagnostics are unchanged on
  purpose (condensing and merging emit none in either lane). The
  lowering corpus lane re-run twice, identical: 12,215 files, 12,215
  checked, with_diagnostics 10,142 — unchanged from installment 3,
  the no-new-diagnostics claim corpus-wide.
- **Metamorphic (TS-21)**: matrix plane census unchanged (321
  mutations, pinned); the corpus shard reproduces P2-15's exact line
  (10,756 mutations); the full corpus run twice with identical counts,
  **179,992 mutations, zero divergences**, now under the ratcheted
  span-elision-only oracle.
- **TS-13**: `assertion-lint: OK`, allowlist untouched.
- **TS-11, deferred precisely**: no shipped path touched — no
  `Cargo.toml` changed (the lowering/pass live in unpublished crates;
  the comparator additions are dev-dep test space), so the dependency
  graph the publish gate checks is bit-identical to the last green run
  and the mechanical argument stands; the clean-fixture `corpus-diff`
  sweep recipe stands as recorded in installment 1.
- **House rules**: every file ≤ 350 after two deliberate splits
  (`lower/text.rs` 341 + `text/condense.rs` 289; the comparator's
  if/for check bodies moved to `s2_support/checks.rs` 149); no
  `mod.rs` under `src/`; ricalco stays `no_std + alloc`
  (wasm32-wasip2 build green); `TextPart` 40 / `TextParts` 24 /
  `TextFacts` 24 size asserts (first guesses held); clippy house
  invocation clean after four first-cut findings (an unused import,
  three needless borrows), the new/touched test targets additionally
  clippy-clean under `--tests`; `cargo fmt --check` clean; the fuzz
  workspace (`s1_lowering`, `folio_parse`) `cargo check`-green.
- **Benches**: none touched, none added — no shipped path gains code.
- **TS-12**: `croquis-consumption.md` regenerated (the new test files
  move the naive consumer counts — the P2-15 deviation-3 convention);
  all `--check` entries green.

### Gaps and owners (what later installments inherit)

- **Comment output**: comments still drop at lowering; the 5
  comment-filler slot units and the text lane's comment-vnode output
  stay S2-invisible. The classes are counted; comment visibility for
  _decisions_ is no longer blocked (this installment's absorption
  solved it), so the future comment op is purely an output feature.
- **Entities**: the 88 entity-skipped templates are the S1
  no-decoding deviation measured over text for the first time; they
  move when S1 or the lowering gains entity decoding, and the counted
  class is the ratchet to watch.
- **`v-pre`**: zero corpus share, but the battery pins the class —
  the legacy parser reads it, S2 defers it; both the text projection
  and the metamorphic exclusions wait on an S2 v-pre story.
- **Interpolation-only regions in `pre`**: merging inside `<pre>`
  keeps bytes but still merges (the legacy codegen grouping never
  checked pre) — recorded as matched behaviour, revisit only if a DOM
  backend needs pre-exact vnode boundaries.
- **Next installment**: `transform_element`/`v_bind`/`v_on` is the
  standing recommendation — it retires `defer.v-bind` (dynamic keys,
  outlet props, forwarded slot props) and the two dynamic-key counters
  that have waited since installment 1, and it is the largest
  remaining defer class. `hoist_static` is the alternative (the first
  S2 **analysis** pass, exercising `Preserved` for real).
