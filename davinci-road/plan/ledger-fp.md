# Davinci — false-positive ledger

> [!NOTE]
> The FP oracle's triage record (assurance doctrine: "Suppression telemetry
> — the FP oracle"). Every vize diagnostic firing on a line users suppressed
> for the analogous upstream rule is an FP candidate; each candidate gets a
> disposition (`fixed` / `justified-with-witness` / `deferred-with-issue`)
> and is never left ambient. An empty candidate section is acceptable only
> alongside scan-scope proof quoted from the tool.

## Suppression-telemetry pilot (P0-13, measured 2026-08-14)

`tools/commands/davinci/suppression-telemetry.rs` lints byte-length-preserving
_defused_ copies (vize honors `eslint-disable` pragmas natively — verified
live — so linting the raw sources would hide exactly the intersection under
measurement) and filters candidates to rule names actually mapped to vize
analogs (mapping source:
`tests/_fixtures/patina-eslint-vue-rule-map.json`, 123 mapped
eslint-plugin-vue rules, plus a verified-core sidecar that is deliberately
empty — no core-ESLint rule has a verified vize analog yet).

Corpus-shard scan, tool output quoted verbatim:

- `scope-proof: files-scanned=130 suppression-comments=1 named=1 bare=0 rules-mapped=123 mapped-seen=0 unmapped-seen=1 fp-candidates=0`
- `unmapped: no-console x1`

**Mapped FP candidates: none** — empty with the scan-scope proof above
(130/130 shard `.vue` files scanned, 123 rules mapped). The single
suppression in the shard's `.vue` sources
(`splitpanes/src/components/splitpanes/splitpanes.vue:689`,
`// eslint-disable-next-line no-console`) names a core-ESLint rule with no
vize analog; per the P0-13 brief it is reported as unmapped, not an error
and not a candidate.

The candidate-detection mechanism itself is proven in CI on the committed
miniature set (`tests/_fixtures/davinci-fpfn`), which plants one mapped
suppression (`vue/no-multi-spaces`) over a firing line and one unmapped
(`no-console`): `scope-proof: files-scanned=4 suppression-comments=2
named=2 bare=0 rules-mapped=123 mapped-seen=1 unmapped-seen=1
fp-candidates=1` (`tests/tooling/davinci-fpfn-pilots.test.ts`).

## FP-1 — `type/require-typed-emits` spans point at the wrong file location

Surfaced by the seeded-defect pilot's baseline identity check (not the
suppression scan): 17 baseline-miss/unexpected pairs on layoutit-grid where
the diagnostic's coordinates failed to track a one-line script insertion —
the reported span drifted by the raw byte delta (+35 columns across a line
boundary) instead of one line, which means the offsets are script-relative
values projected onto whole-file coordinates.

**Witnesses (read at the actual sites):**

- `layoutit-grid/src/components/area/AreaBox.vue`: `defineEmits(['edit'])`
  sits at line 41; the diagnostic reports 9:16–9:37, which is template text.
- `layoutit-grid/src/components/area/AreaButtons.vue`: `defineEmits` at
  line 63; reported at 12:35–12:56.

The rule's fire/no-fire verdict is correct (those `defineEmits` calls are
untyped); the defect is location integrity — under the witness discipline a
diagnostic pointing at unrelated source is triaged here, because users see
squiggles on innocent code. Root: `call.start`/`call.end` from croquis
macro analysis are script-block offsets
(`crates/vize_patina/src/rules/type_aware/require_typed_emits.rs`), while
the output layer converts them against the full-SFC line index
(`crates/vize_patina/src/output/shared.rs`).

**Disposition:** `deferred-with-issue` — offset canonicalization belongs to
the unified diagnostic-channel work (plan P4-6, witness-carrying
diagnostics); tracked there rather than patched ad hoc in one rule, since
`type/require-typed-props` shares the pattern. The miniature CI set is
unaffected (no `defineEmits` usage), so the pilot gate stays exact.
