# Davinci — Assurance Doctrine

> [!NOTE]
> The program creed, stated as engineering mechanisms. Recorded 2026-08-13 as
> charter #21. Every phase gate inherits this page.

## The creed

1. **Never fail.**
2. **Edge cases must not exist.**
3. **No false positives. No false negatives.**
4. **Never regress.**
5. **Every conceivable pattern is tested.**
6. **Tests are strict — nothing passes on partial matching.**

"Never" is not a wish; it is a translation table. A failure mode is either
**impossible by construction** (the type system cannot express it), or
**verified** (a checker rejects it before it ships), or **enumerated and
tested** (it is a case in a matrix with an exact oracle). A failure mode in
none of those three buckets is a design defect, not a bug.

## 1. Never fail — impossibility by construction

- **Illegal states are unrepresentable.** Typed stage enums with no `_` arms —
  adding a variant must break every pass that has to handle it. Ids, spans,
  and keys are newtypes; raw→canonical is a type-level transition; a
  non-canonical artifact cannot reach an optional pass or an emitter.
- **Totality.** Library code does not panic on any input. Malformed source is
  a _represented_ state (`Unexpected`/`Missing` S1 nodes), so "broken input"
  is a normal value flowing through total functions, not an edge case.
  Fuzzing (existing `tests/fuzz` lanes, extended per stage) proves no-crash
  over arbitrary bytes; a fuzz crash fix is complete only with its
  deterministic regression case.
- **What can't be typed is verified.** Debug/CI stage verifiers between passes
  (local, artifact-only, Lean-kernel discipline); `render(S1) == source`
  bytes; incremental ≡ from-scratch over the corpus; the IVM oracle
  (incremental update ≡ full render); Folio `--full` round-trip injectivity.
- **When it fails anyway, it fails loudly and reproducibly.** The operational
  definition of an ICE (charter #30): a file-scoped failure that continues the
  rest of the build, emits an automatic repro package (last-good Folio +
  pipeline string + config, replayable via `vize repro`), and **never**
  degrades to possibly-wrong output — silent degradation is the one behavior
  the creed forbids absolutely.

## No false positives, no false negatives — verdicts are proofs

Rice's theorem says zero-FP _and_ zero-FN over arbitrary programs is
undecidable — but Davinci's domain is not arbitrary programs. Templates are a
small, closed, structured language; the reactivity API surface is finite;
heavy inference exists only at the JS boundary. Domain restriction is what
made Astrée's zero-false-alarm record possible, and it is what makes this
creed credible here. The mechanisms:

- **Three-valued facts, fire-on-proof.** Every semantic fact is
  `proven / refuted / unknown` (the staged-precision shape from the Polonius
  import). Policy, enforced by the rule SDK's types: **error-severity
  diagnostics fire only on `proven`** — zero FP by construction. `unknown`
  never produces an error; it produces silence, or an explicitly-labeled
  hint/suggestion severity that _says_ it is not a proof. A rule cannot
  express "error on maybe".
- **Witness-carrying diagnostics.** An error must carry its witness — the
  concrete fact chain that proves the violation (binding → escape → effect
  edge, with spans via provenance). The witness is machine-checkable against
  the fact base, so a false positive is not a matter of opinion: it is a
  witness that fails verification, caught by the same verifier
  infrastructure as everything else. No witness, no diagnostic. _Migration
  note:_ today's diagnostics (cross-file paths, reactivity `InternalIssue`
  conversions) carry no witness field — the shared witness type and verifier
  arrive with the unified channel (plan P4-6), and phase 4 exits on "every
  error-severity diagnostic carries a verifying witness"; until then legacy
  diagnostics are exempt **by inventory**, never silently.
- **Precision tiers as rule metadata.** Every rule declares
  `exact` (decidable domain — zero FP and zero FN required and testable:
  HTML content model, template CFG, structural rules) /
  `sound` (no-FN over its declared domain) / `complete` (no-FP) /
  `heuristic`. Heuristic rules are barred from error severity by policy, the
  tier renders in the docs, and the declared domain is part of the rule's
  contract — "no FN" always means _no FN within the declared domain_, and
  shrinking the domain silently is a breaking change.
- **Seeded-defect recall — the FN oracle.** FN rates are measured, not
  assumed: the construct matrices generate corpora with _known injected
  defects_ (remove the provider, break the prop type, drop a `key`, race the
  await), and in-domain defect classes require **100% detection** at the
  phase gate. A defect class we claim to catch, we catch every time.
  Measured misses are triaged in the FN ledger
  ([plan/ledger-fn.md](plan/ledger-fn.md); pilot oracle:
  `tools/commands/davinci/seed-defects.rs`, P0-13).
- **Suppression telemetry — the FP oracle.** Real projects carry
  suppressions (`eslint-disable`, waivers). Corpus runs track every vize
  diagnostic on lines users suppressed for the analogous upstream rule, and
  every new suppression Real World Testing users add against _our_
  diagnostics — each is an FP candidate triaged to `fixed` or
  `justified-with-witness`, never left ambient. Candidates land in the FP
  ledger ([plan/ledger-fp.md](plan/ledger-fp.md); pilot oracle:
  `tools/commands/davinci/suppression-telemetry.rs`, P0-13).
- **Divergence is justified or it is a bug.** Charter #23 already bans silent
  divergence from vue-tsc/eslint behavior; under this creed, an unjustified
  divergence is _classified_ — it is either our FP/FN or theirs, and the
  ledger records which, with the witness.

## Never regress — ratchets, not dashboards

A regression is a state transition the system must not be able to make
silently. The mechanisms are one-way:

- **Ratchets.** Budgets and quality metrics only tighten. The repo already
  has the pattern (`tests/_helpers/compat-ratchet.ts`); Davinci generalizes
  it: bench numbers, RSS ceilings, mutation score, rule fact-adoption rate,
  source-map coverage, FP/FN ledger counts — each is a ratchet whose
  loosening requires a charter-level written decision, never a CI tweak.
- **Every fixed bug becomes a permanent test.** Fuzz crashes land with their
  deterministic reproducer (already policy); corpus incidents land as
  fixtures; FP/FN triage outcomes land in the seeded-defect or suppression
  suites. The test outlives the fix and the code it fixed.
- **A deleted failure class stays deleted.** Its reappearance reopens the
  phase that deleted it — not a ticket (standing gate). Remarks-diff over the
  corpus catches _optimization_ regressions the output diff can't see.
- **Baselines are committed artifacts.** Corpus snapshots, bench baselines,
  consumption matrices — diffs against them are reviewed like code, so drift
  is a PR conversation, not an archaeology project.

## Formal methods — surgical, not total

Formal verification is applied where the domain is small, closed, and the
payoff is a load-bearing guarantee — and nowhere else (the Effekt retreat is
the cautionary tale for formalism as a lifestyle). The targets, in order:

1. **S3's executable reference semantics in Lean.** The MIR anti-lesson says
   pin down what an Impeto effect _means_ before optimizing; the React-tRace
   precedent says make the semantics executable and differential-test the
   optimized implementation against it. Writing that reference in Lean makes
   it simultaneously the spec, a test oracle, and a proof substrate.
2. **Theorems on the small models.** The reactivity lattice's lattice laws
   (classification monotonicity, join correctness); effect-grouping
   preserves the dependency edge set; the IVM linearity claim (a keyed
   `v-for` patch plan ≡ recompute-from-scratch on the reference semantics).
   Small, closed statements — exactly Lean's sweet spot.
3. **An independent Folio checker.** Because Folio round-trips, stage
   invariants can be verified out-of-process by a second implementation that
   shares no code with the compiler (the Lean4Lean discipline) — potentially
   _written in Lean_, parsing folios and checking S2/S3 invariants in CI.
4. **Decidable checkers proved total.** The HTML content-model checker and
   region well-formedness are finite, decidable domains where `exact`
   precision (zero FP/FN) is a provable property, not an aspiration.

Rust-side proofs (e.g. Kani/Creusot on `unsafe` islands) are evaluated
case-by-case where charter #22's complexity license gets exercised.

## 2. Edge cases must not exist — elimination by enumeration

An "edge case" is an input region nobody enumerated. The countermeasure is
owning the input space:

- **Construct matrices.** Each dialect's surface constructs are a finite,
  documented set (elements × directives × modifiers × slots × control flow ×
  script binding kinds). New construct ⇒ new matrix row ⇒ the combination
  suites regenerate. A construct without a matrix row cannot merge.
- **Combinatorial coverage.** Pairwise at minimum, exhaustive where the
  product is small (directive × position × modifier). Generated fixtures, not
  hand-picked examples — hand-picked sets are where edge cases hide.
- **Property, metamorphic, differential.** Properties (idempotence,
  parse-preservation — Glyph's four corpus properties generalized to every
  surface); metamorphic SFC mutations with folio-equivalence oracles;
  differential oracles against reference behavior (Vue/vue-tsc parity, the
  Polonius-style naive rule evaluator for fact groups, the S3 reference
  interpreter).
- **The corpus is the floor, not the ceiling.** 134 real projects prove
  absence of regressions on code that exists; the matrices and properties
  cover code that doesn't exist yet.

## 3. Every conceivable pattern is tested — the tier ladder

| Tier         | Unit                        | Oracle                                                                        |
| ------------ | --------------------------- | ----------------------------------------------------------------------------- |
| Fixture      | one construct, one stage    | full normalized Folio snapshot (exact)                                        |
| Pass         | one pass via `davinci-opt`  | full normalized Folio snapshot (exact)                                        |
| Verifier     | invalid artifact            | exact diagnostic (code + span + full message, pinned to the canonical locale) |
| Matrix       | construct combinations      | generated expected outputs, exact                                             |
| Property     | generated inputs            | invariant holds — no exceptions list                                          |
| Metamorphic  | mutated SFC pairs           | folio equality modulo declared normalization                                  |
| Differential | vs reference implementation | exact agreement or explicit, reviewed waiver                                  |
| Behavioral   | compiled output, mounted    | scripted interaction trace equality (sprout-style)                            |
| Corpus       | 134 projects                | byte-identical or waivered; ledger empty at phase exit                        |
| Editor       | LSP scenarios               | exact protocol-level expectations, multi-client                               |

Every phase's exit gate names which tiers it extends. A feature testable in a
tier but not tested there is untested.

## 4. Strict oracles — no partial matching

- **Locale-stable exactness.** Diagnostic-text oracles pin the canonical
  locale (`en`) plus stable message ids; i18n catalogs (charter #42) get
  their own per-locale completeness checks and snapshot fixtures, so
  translation work never loosens — and never breaks — the canonical oracles.
- **Exact equality only.** Assertions compare whole normalized artifacts:
  full Folio snapshots, byte-identical outputs, structural equality on typed
  values. **Banned in test code:** substring/`contains` assertions, regex
  loosening, partial JSON matching, prefix/suffix checks, count-only checks,
  and threshold assertions where exact values are computable. If output is
  nondeterministic, the fix is normalization in the printer (stable ids,
  sorted maps), never a looser assertion.
- **Targeted assertions supplement, never replace.** The rustc FileCheck
  practice is adopted _under_ this rule: a pass test's oracle is the full
  normalized folio; targeted structural assertions may document the specific
  property the pass claims, in addition — an exact structural match on a named
  sub-object, never a substring.
- **Oracle truth.** An exact assertion of a wrong expected value is worse than
  none — it pins the bug as correct (this happened: a canon test normalized a
  virtual-path leak into its expected message and froze the leak). Expected
  values must be justified — against Vue/TypeScript reference behavior, a
  spec, or a documented decision — not merely recorded. Snapshot review asks
  "why is this output _right_?", not "did it change?".
- **Rebaseline discipline.** Snapshots are reviewed contracts
  (language-engineering-practices). A PR refreshing more than a handful of
  snapshots must explain every group of diffs; bulk-accept is prohibited.
- **Tests are themselves tested.** Mutation testing (`cargo-mutants`) runs on
  Davinci crates: a surviving mutant in a stage, pass, or verifier is a
  missing or lax test and blocks the phase gate. This is how "strict" is
  measured instead of asserted.
- **Assertion lint.** The banned-pattern list is enforced mechanically (test
  lint in CI), the same way `clippy.toml` bans `std::string::String` — not by
  review vigilance.

## Enforcement summary

Phase 0 adds: the assertion lint, the mutation-testing baseline, matrix
generators for the existing surface, and the normalized-printer rules that
make exact snapshots sustainable. Every later phase inherits: verifier +
matrix + property + corpus gates, empty waiver ledgers, mutation score held,
and the standing rule that a deleted failure class stays deleted — a
regression reopens the phase, not a ticket.
