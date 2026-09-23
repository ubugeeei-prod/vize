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

**Disposition:** `fixed` (P4-6c, 2026-09-22). The root cause was one
frame confusion shared by both rules: croquis' lint analysis measures macro
calls in its script frame (`<script>` content, `\n`, `<script setup>`
content when both exist), and the rules reported those offsets through the
template frame. `vize_patina::output::frame::ScriptFrame` now maps the
script-analysis frame to the file block by block, validated through
`vize_s0::{SourceRoot, SourceBlock}`, and both the rule path
(`LintContext::report_in_script`) and the native type-aware static warnings
report through it; a range no block contains is refused, never guessed.

Re-measured on the same corpus shard (130 files, 49 class-a and 130 class-b
injections, `seed-defects.rs --fixtures <shard> --assert` fed by a
`vize_patina` library harness in place of the CLI, identical before and after
the fix): **baseline-shift pairs 17 → 0** (16 layoutit-grid + 1 splitpanes,
all `type/require-typed-emits`, including both witnesses above — `AreaBox.vue`
now reports 41:1–41:22 and `AreaButtons.vue` 63:1–63:22, exactly the
`defineEmits(['edit'])` calls).
Permanent tests: `crates/vize_patina/tests/script_frame_spans.rs`
(template-first, script-first, `<script>` + `<script setup>`, multibyte +
CRLF, rendered line/column). TS-9 lint snapshots changed exactly the eight
FP-1 spans they carried (elk ×4, npmx.dev ×1, reka-ui ×2, ant-design-vue ×1),
each re-derived from the pinned fixture source.

## FP-2 — `vue/permitted-contents` flagged conforming label and select content

Surfaced by the P4-11a upgrade, which re-derived the rule from the pinned
WHATWG snapshot (`crates/vize_patina/src/html_content_model/whatwg.tsv`).
The hand-written rule treated `label`, `select` and `textarea` as containers
that forbid interactive descendants, and restricted `select`/`optgroup`
children to `option`.

**Witnesses:**

- `elk/app/components/account/AccountHeader.vue:236:11` and
  `elk/app/pages/settings/profile/appearance.vue:180:11`: a `<textarea>` inside
  a `<label>` is that label's labeled control; the label content model forbids
  only other labelable descendants and nested labels (`forms.html#the-label-element`).
- `<select><div>` and `<select><hr>`: both are select element inner content
  since the customizable-select change (index "List of elements", `select`
  children column).

**Disposition:** `fixed` — the rule now reports only proven violations of the
spec-derived content models; the corpus lint snapshots (elk, misskey,
npmx.dev, nuxt-ui, reka-ui) carry 24 of the 26 prior reports at the same spans
and drop exactly the two witnesses above. The 107 new reports are triaged
`justified-with-witness`: 106 flow content inside `<button>` (content model
phrasing, `form-elements.html#the-button-element`) and one `<tr>` directly in
`<table>` (`elk/app/components/report/ReportModal.vue:190:9`; the parser
inserts an implied `<tbody>`, `parsing.html#parsing-main-intable`).

## FP-3 — `html/cross-component-nesting` reports a child branch the call site cannot take

Surfaced by the P4-11b corpus triage (`vize lint --cross-file` over elk,
misskey, npmx.dev, nuxt-ui and reka-ui: 9 findings, all in misskey). The
composed check re-reads every child template in the parent's open-element
chain and, like the per-file checker, does not evaluate `v-if` conditions:
every branch is assumed reachable.

**Witnesses:**

- `misskey/packages/frontend/src/pages/drive.file.info.vue:39:5` and `:45:5`:
  `<MkKeyValue>` inside `<button>`; `MkKeyValue.vue:13:3` renders
  `<button v-if="copy">`, and neither call site passes `copy`, so the nested
  button never renders there. The same usages are still non-conforming — the
  child's root `<div>` (`MkKeyValue.vue:7:1`) inside `<button>` is reported
  alongside, and that report is justified.

**Disposition:** `fixed`. Composition now prunes a child `v-if="prop"`
branch at a usage that leaves `prop` unpassed, when the child's `<script
setup>` proves the absent value falsy. `MkKeyValue` declares
`withDefaults(…, { copy: null })`. The rule and its exactness conditions are
in [phase-4-records/p4-11b.md](./phase-4-records/p4-11b.md) ("Prop-sensitive
pruning"). Re-running the corpus drops exactly the two witnesses above. The
other 7 findings are `justified-with-witness`: `<div>` roots inside `<button>`
(2, the root sites above) and inside `<span>` (3,
`src/ui/_common_/statusbar-{federation,rss,user-list}.vue:16:4` through
`MkMarqueeText.vue:7:1`; both content models are phrasing), and a `<canvas>`
and an `<img>` carrying `tabindex` inside `<a>`
(`src/pages/admin-file.root.vue:11:5` through `MkImgWithBlurhash.vue:17:3`
and `:29:3`; the `a` content model forbids descendants with `tabindex`,
`text-level-semantics.html#the-a-element`).
