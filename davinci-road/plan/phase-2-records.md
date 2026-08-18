# Phase 2 — Task records

> [!NOTE]
> What each landed phase-2 task actually measured, decided and left open. The
> **contracts** are in [phase-2-tasks.md](./phase-2-tasks.md) and the
> phase-level record — the re-cut, the phase-1 carry-ins, the TODO index and
> the exit gate — is in [phase-2.md](./phase-2.md). Records live in a third
> file for the reason the first two split: the repository's 350-line
> source-length budget (`tools/moon/cmd/source_file_lengths --max-lines 350`),
> which plan files are not exempt from, and a record grows with every task.

## P2-1

**`vize_davinci` core types. Landed 2026-08-19; every acceptance clause met,
one contract step deviated from deliberately.**

### `NodeId`: `NonZeroU32`, not a reserved sentinel

Both give `Option<NodeId>` the required 4-byte layout — an S2 op holding
several optional child references pays 4 bytes per slot, not 8. `NonZeroU32`
wins on two counts: it makes "no such node" **unrepresentable inside `NodeId`
itself** (with a reserved-sentinel `u32`, every consumer has to remember that
`NodeId(0)` is not a node and nothing stops one from building it), and the
niche is stable Rust — reserving a range on a plain `u32` needs
`rustc_layout_scalar_valid_range`, which is not.

The cost is that the raw value and the index differ by one. That arithmetic is
single-sourced in `NodeId::from_index` / `NodeId::index` and appears nowhere
else; `Debug` and `Display` print the **index** (`NodeId(0)`, `%0`) because
that is what folio pages and diagnostics show. Exhaustion returns `None` rather
than wrapping or panicking: a stage with more than `u32::MAX - 1` nodes in one
artifact should emit a diagnostic, not abort.

### `SideTable`: sparse only, with the trigger written down

P1-10 proved the residency question is not free — `vize_carton::{Box, Vec}` are
`oxc_allocator`'s, whose const assertion **rejects `Drop` payloads**, and that
assertion caught two real violations during P1-10. So the two forms are not
interchangeable, and P2-1 lands the sparse one:
`vize_carton::FxHashMap<NodeId, T>`, an ordinary non-arena scratch structure
free to hold a `T` that owns heap memory.

The dense arena form (`vize_carton::Vec<'a, Option<T>>`) is **documented, not
built**, per the contract. Its trigger is three conditions that must all hold,
measured rather than assumed: high occupancy (a fact for a majority of nodes,
below which the `Option<T>` slots cost more than hash entries), a `Drop`-free
`T`, and a lookup-dominated access pattern. The measurement belongs to
whichever pass proposes it, in its own PR — a blanket switch is not a valid
form of that change.

**Iteration order is a trap the API names.** Hash-map order is unspecified and
folio pages require sorted map iteration (`folio-format.md`), so `iter()` says
in its docs that it is not the printing path and `sorted_entries()` — which
allocates, visibly, at the call site — is.

**Naming:** the membership predicate is `contains_id`, not `contains`. A keyed
table's membership question is about the id, the way `HashMap::contains_key`
says so. This also clears TS-13's `contains` finding **at the source**, which
is the only acceptable resolution: the allowlist's own header says new test
code must never be added to it.

### `Diagnostic`: owned, `'static`, with the witness slot real

`Span` and nothing else for coordinates — `Position` does not exist since P1-4,
and line/column are derived at rendering time from `LineIndex`. Every field is
owned, enforced by a `const` block asserting `Diagnostic`, `DiagnosticPart` and
`Witness` are `'static`: the P1-11 arena/cache contract, since diagnostics
cross the compile boundary by definition — they outlive the compile, get
collected across a batch, and get cached. Message text is owned, the deliberate
P1-10 exception `CompilerError::message` already carries.

The **witness slot is typed rather than absent**. `assurance.md`'s migration
note is that legacy diagnostics are exempt "**by inventory**, never silently",
so `Witness::LegacyExempt(String)` names the producer: the exemption is an
entry in a list rather than a missing field. `satisfies_witness_law()` is the
predicate P4-6 will gate on and an inventory can count today; nothing enforces
it yet, by design.

`Severity` is included although the contract did not name it, because
`assurance.md`'s no-error-on-maybe rule and P4-6's law are both stated in terms
of error severity — a diagnostic type that cannot express severity cannot carry
either.

### Deviation: which size asserts carry the pointer-width guard

The contract says node-size asserts on all three types, "each guarded
`#[cfg(target_pointer_width = "64")]`". **They are not all guarded, and the
reason is the guard's own rationale.** That guard exists in `vize_relief`
because those figures are 64-bit footprints of pointer-containing structs
("the wasm32 build is 32-bit",
`crates/vize_relief/src/relief/elements.rs:31-36`). `NodeId` holds a `u32` and
no pointer; `Severity`, `Stage` and `PartKind` are single-byte tags. Their
footprints are identical on every target, so guarding them would only stop the
wasm32 lane **P2-14 makes required** from checking them — and the `NodeId`
niche is the property most worth checking there.

So: `NodeId`, `Option<NodeId>` and the three tag enums assert unconditionally;
`Diagnostic` (88 bytes) and `DiagnosticPart` (40) carry the guard, because they
contain pointers. The 88 bytes are `Span` (8) + the two tags + an owned
`String` message (24) + the parts `Vec` (24) + the witness slot (24) — and that
last number is why a real witness type must be boxed rather than inlined when
P4-6 lands it.

### Acceptance

- `cargo test -p vize_davinci` green: 20 new unit tests (31 with the existing
  folio suites).
- `cargo build -p vize_davinci --target wasm32-wasip2` green — the P0-10
  acceptance, kept; the *required* lane lands at P2-14.
- TS-11 empty, **proved rather than argued**: `cargo tree -i vize_davinci
  --workspace` lists no reverse dependencies, so no compile path can observe
  these types and no output byte can move.
- TS-13 green with **no allowlist entry added**.
- `cargo clippy -p vize_davinci --all-targets` clean, including the
  `disallowed_macros` rule that rejects `std::format` (the tests use
  `vize_carton::cstr!`).

## P2-12a

**Phase-start baselines and pinned targets. Landed 2026-08-19 at rev
`232870a8`; five of six steps met, the corpus `--check` clause carried with a
plan finding.**

### The probe

`crates/vize_atelier_core/src/walk_probe.rs` — 19 sites (15 visit sites, 4
stage-walk sites), tabulated in the module docs. A **visit** is counted where a
stage's descent dispatches on a template node's kind to decide how to continue;
a **walk** at a stage's root entry only.

**Why 19 sites and not 4.** The four obvious funnels — `traverse_node`,
`generate_node`, SSR `process_child`, Vapor `transform_children` — are not the
whole descent, and each gap was found by a number that read wrong rather than
by inspection:

- `generate_root_node` handles the single-root case without passing through
  `generate_node`. Instrumenting only the funnel reported `stress-wide` DOM
  codegen as **0 visits** while codegen was emitting a 100-attribute element.
- `generate_v_once_child` emits Text and Interpolation children directly.
- SSR builds component-slot children through a **second** descent
  (`vnode_child_expression`) that never reaches `process_child`.
- Vapor lowering has **six** child-list walkers, not one. Instrumenting only
  `transform_children` reported `stress-deep` at 9 lowering visits against 72
  transform visits — a 20× understatement.

The corrected Vapor numbers are the phase's most useful finding; see below.

### Headline numbers

Fused compile, ladder fixture, default options, `walks / visits`:

| fixture       |     DOM |      SSR |    Vapor | transform component |
| ------------- | ------: | -------: | -------: | ------------------: |
| small         |  2 / 11 |   2 / 16 |   2 / 25 |                   8 |
| medium        |  2 / 62 |  2 / 118 |  2 / 102 |                  33 |
| large         |  2 / 86 |  2 / 106 |  2 / 127 |                  57 |
| stress-deep   | 2 / 134 |  2 / 144 |  2 / 256 |                  72 |
| stress-wide   |   2 / 3 |    2 / 4 |    2 / 4 |                   2 |
| stress-interp | 2 / 1102 | 2 / 2002 | 2 / 3102 |                1001 |

Three readings matter for the phase:

1. **`walks = 2` on every row, and the transform column is identical across all
   three backends.** Every backend pays the same transform traversal in full
   and then walks the tree again — [motivation.md](../motivation.md)'s
   duplicate-work fault line in the traversal dimension rather than the parse
   dimension.
2. **Vapor lowering re-walks the same children 2.1–2.6×** (`stress-deep` 184
   lowering visits against 72 transform visits; `stress-interp` 2101 against
   1001), because six distinct child-list walkers each descend into an element
   whose children are dynamic. Region-owning `ui.for` / `ui.if` (P2-5a) and
   fusion (P2-2) aim exactly there.
3. **`stress-wide` is a floor at 3–4 visits on every backend.** Attribute
   width is prop work, not traversal work — which is why it is the target
   table's named control row.

The full per-stage breakdown, the reproducing command, the two-run determinism
proof and the **named exclusion list** are in
[walk-baseline.md](./walk-baseline.md).

### What was pinned

- `budgets.toml [traversal]` — 18 entries keyed `<backend>_<fixture>`,
  `{ walks, visits }`, gated **exactly**. The section docs make the
  machine-independence argument the way the `allocs` field docs were rewritten
  at P1-13; unlike `wall_p50_ns` there is no "0 means unrecorded" state.
- `budgets.toml [target.phase-2]` — `phase_start_rev =
  232870a83506cf3312cc8ef02e91c8b73ac12d2b`, `phase_start_date = 2026-08-19`,
  `dom_walks_max = 1` (the fused S2 path makes one walk where the pipeline
  makes two), `dom_visits_ratio_max = 0.80` with `stress-wide` named as the
  excluded control row, `dom_compile_allocs_ratio_max = 1.00` (S2 may not be
  paid for in allocations), `ssr_visits_ratio_max` / `vapor_visits_ratio_max =
  1.00` (held, not improved — those backends stay on the legacy path until
  P3), and `wall_time = "report-only"`.
  **Review point:** the maintainer sets these numbers. They are proposed from
  the measurement above; CI owns only their existence, non-zero-ness, a 40-hex
  rev and an ISO date, because the assurance doctrine forbids choosing them
  later to fit the result.
- `crates/vize_atelier_{dom,ssr,vapor}/tests/davinci_walk_baseline.rs` — the
  same numbers pinned as ordinary integration tests, so they run in the
  default `cargo test --workspace` lane (the P1-5/P1-7 counter-law shape).
  Each asserts the whole table at once, so a re-record prints every row rather
  than stopping at the first drift.

### The gates, and proving them

`tests/tooling/davinci-traversal-budgets.test.ts` is a new suite rather than
more of `davinci-budgets.test.ts`, because either file alone would breach the
350-line source budget. It reconciles in both directions — the backend domain
is derived from the crates that ship a `tests/davinci_walk_baseline.rs`
recorder × the harness `LADDER`, so a recorder without ceilings and a ceiling
without a recorder both fail — and a third test asserts every ceiling
**equals** the `BASELINE` table its recorder pins Rust-side, so the two
committed copies of the number cannot drift apart.

All three were **proven, not assumed** (the P0-7 pattern): a ceiling edited off
its Rust pin, an id with no recorder cell, and a zeroed target value each fail
exactly one test and leave the others green.

`tests/tooling/davinci-budgets.test.ts`'s "one single-line inline table" check
became section-scoped in the same change, since `[traversal]` reuses that shape
with a different field set.

### Plan finding: the corpus `--check` clause is not evaluable

P2-12a's acceptance clause "corpus-coverage `--check` green with scope proof
(TS-12)" **cannot be run by anyone today**, and this task is where that
surfaced rather than a scope the task got wrong (plan README: the plan is
code). `tools/davinci/corpus-coverage.mjs --check` byte-compares the committed
report against a fresh scan of the **hydrated** corpus;
[corpus-coverage.md](./corpus-coverage.md)'s own header already records that
"the `--check` staleness gate can only join
`tests/tooling/davinci-matrices.test.ts` once CI hydrates the full corpus", so
neither CI nor a normal working tree can run it — an unhydrated tree scans 0 of
142 projects and reports drift that means nothing.

**What was done instead**: the audit's substance, against the committed
142/142-hydrated report. Every S2 op P2-5a plans has real-project instances —

| S2 op           | corpus evidence                                                                   |
| --------------- | --------------------------------------------------------------------------------- |
| `element`       | native start tags: 178 334 sites / 137 projects                                   |
| `component`     | 176 418 / 134                                                                     |
| `ui.slot`       | slot elements 13 745 / 117, plus 22 765 `v-slot` occurrences with no taxonomy row |
| `ui.if`         | `v-if` 31 734 / 126, `v-else-if` 10 898 / 84, `v-else` 5 547 / 110                |
| `ui.for`        | 9 926 / 128                                                                       |
| `ui.model`      | 22 693 / 126, with 532 modifier sites / 47 projects                               |
| `vue.directive` | custom directives 2 534 / 72                                                      |

— **with one exception: `mathml` is 0 sites across 0 projects**, so a MathML
element kind is **not represented — matrix fixtures only**, and P2-15's
generated plane is where it has to be covered. Thin-but-present classes to keep
in view when P2-9 migrates their transforms: `v-pre` (4 sites / 3 projects),
`v-memo` (10 / 5), `v-cloak` (16 / 2), `v-once` (590 / 5) and mouse-button
modifiers (54 / 12).

**The residual clause** — running `--check` itself — carries to the phase-2
exit gate's C-14 line, where it belongs with the corpus operations the gate
already schedules on a hydrated checkout.
