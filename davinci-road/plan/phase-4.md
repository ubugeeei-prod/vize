# Phase 4 — Consumer Convergence (provisional decomposition)

> [!WARNING]
> Provisional; re-cut at phase-3 exit. The widest phase — expect task splits.
> Suites referenced as TS-n from [test-suites.md](./test-suites.md).

## TODO index

- [ ] P4-1 Fact engine query API
- [ ] P4-2 Fact-group α/β split
- [ ] P4-3 Croquis tracker migration waves
- [ ] P4-4 Orphan productize-or-gate decisions
- [ ] P4-5 One virtual-language projection
- [ ] P4-6 Witness + precision-tier diagnostics SDK
- [ ] P4-7 Markup facade on S2
- [ ] P4-8 Rule migration waves (345 files)
- [ ] P4-9 Complexity facts + rules
- [ ] P4-10 App-level fact providers
- [ ] P4-11 HTML conformance facts
- [ ] P4-12 Glyph on S1 + style spec
- [ ] P4-13 Musea onto S0/S1
- [ ] P4-14 Diagnostics rendering + i18n
- [ ] P4-15 FP/FN oracles at full scale
- [ ] P4-16 JS plugin SDK spike
- [ ] P4-17 Phase exit

---

**P4-1 Fact query API.** Static demand declarations (const fact-group sets
per consumer), debug undeclared-access detector (TS-35), `FactManager` with
post-hoc `Preserved` sets + named preservation groups (LLVM new-PM import) and
the debug recompute-and-compare mode; stratification makes demand cycles
unrepresentable (Swift anti-lesson). _Accept:_ TS-35; detector self-test.

**P4-2 α/β split.** Each fact group defines (α serialized entries with
explicit `export` fn, β in-memory index rebuilt on demand) — Lean environment-
extension import; α is versioned independently and feeds the P5 summary
contract. _Accept:_ serialization round-trip tests; α-schema docs per group.

**P4-3 Tracker migration waves.** Order: `bindings` → `reactivity` (merges
with P3-2 lattice) → `component_usages` → `undefined_refs` → `unused_bindings`
→ `EffectGraph` → `ProvideInject` → `RaceCondition`. Each wave: declarative
rule spec + naive differential evaluator (TS-34), consumers switched from
struct-field reads to demands, consumption matrix (TS-12) updated. _Accept:_
per-wave TS-34 agreement over a corpus shard; TS-11 empty.

**P4-4 Orphan decisions.** With TS-34 evidence per orphan: `ProvideInject` →
cross-file pairing rules; `RaceCondition` → async-setup race rules;
`EffectGraph` → already consumed by Vapor (P3-6) + Doctor. Any product
failing corpus soundness gets demand-gated to zero cost (charter #5), with
the decision recorded in the charter. _Accept:_ consumption matrix shows no
computed-but-unconsumed groups (TS-12).

**P4-5 One projection.** Single S2-based virtual-language projection replaces
`vize_canon/src/virtual_ts/` and `vize_maestro/src/virtual_code/`; one
mapping model (`VizeMapping` unified) serves the Corsa/tsgo API, the
content-mapper protocol (`vize content-mapper`), and Maestro; diagnostics
assembled in **one post-pass** over finished `Vec<Diagnostic>` (kills the
session-vs-CLI dual path — the known incident class). _Accept:_ TS-40; TS-25
differential lane old-vs-new projections during migration; both old
generators deleted at task end. The
[current-projection baseline](./ts40-projection-differential.md) freezes the old
side without claiming that this task has started.

**P4-6 Witness + precision SDK.** Rule SDK types enforce: error severity ⇒
`proven` verdict + witness (fact chain with spans); tiers
`exact/sound/complete/heuristic` declared per rule, rendered in docs;
heuristic barred from error severity at the type level. Witness re-checking
in CI (TS-36). _Accept:_ TS-36; a canary rule that tries error-on-unknown
fails to compile.

**P4-7 Markup facade on S2.** `vize_patina/src/markup.rs` projects from
Disegno (zero-copy) instead of relief/oxc-JSX duality; `ir.rs` document kinds
map to input dialects. _Accept:_ TS-11 lint surface empty; facade benches
(`markup_ir_bench`) hold.

**P4-8 Rule waves.** Migration order driven by the rule-parity matrix
(P0-8): neutral-core candidates first (largest shared win), dialect-bound
second, container-bound last; per-wave corpus lint-agreement gates (TS-39);
fact demands replace direct croquis reads (26/345 → tracked upward each
wave). _Accept:_ TS-39 per wave; TS-12 matrices current; final SFC/JSX
convergence per charter #7's litmus.

**P4-9 Complexity facts.** Template-CFG cyclomatic + cognitive metrics over
S2 regions; cross-file attribution via the component graph (metric definition
decided here with corpus distributions — open question resolves); Patina
rules + Doctor findings; `vize_curator/src/complexity` absorbed. _Accept:_
metric spec doc + TS-34-style spec/impl agreement; rule ships with tier
`exact`.

**P4-10 App-level providers.** Provider contract (decided here: plug-in
kind vs privileged fact-API consumer — open question); in-tree providers:
Vue Router (typed route params at `router.push`/`<RouterLink>`) and Nuxt
(`definePageMeta`, route tree from `pages/`), generalizing
`vize_maestro/src/ide/ecosystem.rs`. _Accept:_ route-typing diagnostics on
matrix fixtures; TS-40 projection includes route param types.

**P4-11 HTML conformance.** Content-model fact tables (spec-derived,
committed as data), per-file checks upgraded, **composed cross-component
check** via render-tree facts (`<p>`-parent × component-root-`<div>` class of
bugs); tier `exact` — provably total checker (formal-methods target #36-4).
_Accept:_ seeded-defect classes for nesting violations at 100% recall
(TS-37); zero FP on corpus (TS-38 triage).

**P4-12 Glyph on S1 + style spec.** Blank-slate style discussion → written
`style-spec.md` (every decision as a fixture, TS-41) → reimplementation on
lossless S1 (byte scanner `crates/vize_glyph/src/template/formatter.rs`
deleted); pug via the S1 pug dialect. _Accept:_ TS-5 four properties green
with empty waivers; TS-41 spec fixtures exact; churn-vs-old report attached
for review (not gated).

**P4-13 Musea onto S0/S1.** `vize_musea/src/parse.rs` hand scanner replaced
by S0 block splitting + S1 trees; `<art>`/`<variant>` as custom blocks.
_Accept:_ musea fixture suite (TS-1) green; TS-11 art surface empty.

**P4-14 Diagnostics rendering + i18n.** rustc/Elm-grade renderer on the
unified channel (labels, help, fix suggestions, witness-derived "why"
expansion); i18n catalogs (en/ja/zh, patina precedent) extended to all
diagnostics; `--explain <code>` pages generated from rule metadata.
_Accept:_ renderer snapshot fixtures (exact, per locale); every error code
has a catalog entry (CI completeness check).

**P4-15 FP/FN full scale.** Seeded-defect classes extended to the full
in-domain matrix (every `exact`/`sound` rule has generated defect fixtures);
suppression telemetry across the whole corpus each CI week. "Zeroed" means
**measurably closed, not empty-by-hope**: TS-37 at 100% recall per in-domain
class, and TS-38 with zero _untriaged_ candidates — every candidate resolved
to `fixed` or `justified-with-witness`. _Accept:_ those two numbers, in CI.

**P4-16 JS plugin spike.** API-shape decision (serialized visit batches vs
proxies; worker vs sync napi; JS-side demand declaration) proven with one
real custom rule through `vize_vitrine`'s napi lane; batched node visits.
_Accept:_ spike rule runs deterministic + cost-attributed; decision recorded
(GA in P6-7).

**P4-17 Phase exit.**

- [ ] TS-40 check parity; TS-39 lint agreement; TS-5 + TS-41 glyph gates
- [ ] Consumption matrix: every computed group ≥1 consumer or gated (TS-12)
- [ ] TS-36 witnesses verify; TS-37 100% recall per class; TS-38 zero untriaged candidates
- [ ] canon/maestro projection duplicates + glyph byte scanner + musea hand parser: deleted
