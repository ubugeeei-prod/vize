# Davinci — false-negative ledger

> [!NOTE]
> The FN oracle's triage record (assurance doctrine: "Seeded-defect recall —
> the FN oracle"). Every defect class the seeded-defect generator injects is
> asserted by **identity, not count**: the manifest records each injection's
> file, span, and expected rule id, and the `--assert` mode of
> `tools/commands/davinci/seed-defects.rs` compares the exact diagnostic set against
> it. A measured miss lands here with a disposition (`fixed` /
> `justified-with-witness` / `deferred-with-issue`) and is never left
> ambient.

## Pilot scan scope (P0-13, measured 2026-08-14)

Tool output quoted verbatim (`scope-proof` lines are printed by
`tools/commands/davinci/seed-defects.rs`); the corpus-shard run is local/nightly,
the committed miniature set runs in CI via
`tests/tooling/davinci-fpfn-pilots.test.ts`.

- Corpus shard (`splitpanes+layoutit-grid+cssgridgenerator`):
  `scope-proof: files-scanned=130 class-a-eligible=49 class-a-injections=49 class-b-injections=130`
- Matrix stubs (`matrix-gen`, element-kind × directive plane):
  `scope-proof: files-scanned=90 class-a-eligible=0 class-a-injections=0 class-b-injections=90`
  (stubs are template-only, so class (a) has no script binding to rename)
- Committed miniature set (`tests/_fixtures/davinci-fpfn`):
  `scope-proof: files-scanned=4 class-a-eligible=3 class-a-injections=3 class-b-injections=4`

## FN-1 — class (a) undefined-template-ref: `vue/no-undefined-refs` is unreachable

**Measured recall:** 0/49 on the corpus shard, 0/3 on the miniature set —
identity-verified, every miss listed by exact location and identifier.

**Witness (registration gap, verified against the sources and the CLI):**
the rule is implemented
(`crates/vize_patina/src/rules/vue/no_undefined_refs.rs`), exported, and
listed in `SEMANTIC_TEMPLATE_RULES`
(`crates/vize_patina/src/linter/engine/rule_sets.rs`), but no
`RuleRegistry` preset constructor (`crates/vize_patina/src/rule.rs`) and no
opt-in registration path (`register_opt_in` in
`crates/vize_patina/src/rules/vue.rs`) ever instantiates it. Config-enabling
`"vue/no-undefined-refs": "warn"` is a no-op: `with_additional_rules` only
adds rule _names_ to the enabled set, it cannot summon an unregistered rule
instance (verified with a live `vize lint` run; a config-enabled opt-in rule
fires, this rule does not).

The P0-13 plan sentence "current Patina must flag 100% of seeded class-(a)
instances" is therefore falsified by the pilot — which is precisely the
assumption-testing job the oracle exists to do. The identity-assertion
_mechanism_ is proven green both ways in CI (a synthetic diagnostic set
matching the manifest passes; a same-count wrong-location set fails listing
the exact miss).

**Disposition:** `justified-with-witness` for default-preset recall;
config-enable is fixed.

- Default presets still do not instantiate the rule, so the seeded-defect
  pilot remains 0/3 (`tests/_fixtures/davinci-fpfn/expected/assert-report.json`).
  Putting it on a default preset is a separate FP-audit change.
- `#4636` / `fix/patina-undefined-refs-dispatch` registers the rule on the
  opt-in path. `"vue/no-undefined-refs": "warn"` now instantiates and runs
  the rule. The engine rule-name set is no longer a dead gate entry.

The CI expectation still pins the measured default-preset 0/3. Flip that
pin in the same change that adds the rule to a default preset.

## FN-2 — class (b) unused-binding: `unused_bindings` has no lint consumer

**Measured recall:** 0/130 on the corpus shard, 0/90 on the matrix stubs,
0/4 on the miniature set.

**Witness:** `vize_croquis` computes `unused_bindings`
(`crates/vize_croquis/src/croquis.rs`), but no lint rule consumes it —
`vue/no-unused-vars` covers only `v-for`/`v-slot` variables
(`crates/vize_patina/src/rules/vue/no_unused_vars.rs`). This is the gap the
P0-13 plan documents by design; the pilot turns it into a measured number.

Caveat recorded for the future flip: the seeded identifier
(`__davinci_seeded_unused`, mandated by the P0-13 spec) is
underscore-prefixed, and existing unused-checks treat `_`-prefixed names as
intentionally unused — when a consumer lands, the seed name must be
revisited together with this entry.

**Disposition:** `deferred-with-issue` — the consumer arrives with the rule
SDK / fact-channel work (assurance doctrine, precision tiers); explicitly
not a phase-0 gate per the P0-13 plan text.
