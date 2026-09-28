# Native-only Acceptance and Emitter Selection (2026-09-27)

This companion belongs to the canonical
[decision record](./2026-09-27-level-restructure.md). It tracks
[#6853](https://github.com/ubugeeei-prod/vize/issues/6853) and
[#6854](https://github.com/ubugeeei-prod/vize/issues/6854).
The [paired issue evidence](https://github.com/ubugeeei-prod/vize/issues/6853#issuecomment-5855250565)
records this same decision and the private validation scope.

## Verified gap

At main `c6916de9c630bf16cb3f5107dfedb16d8f8bd3d4`, the production
benchmark summary labeled every `selected_lane == "accepted"` observation
as native for the requested backend. That counter proves emitter selection,
not the origin of all facts that the emitter consumes:

- `vize_atelier_dom/src/compile/croquis_facts.rs` permits a projectable
  Croquis summary and projects its legacy binding/reactivity facts into the
  L2 `BindingTable`.
- `vize_atelier_sfc/src/compile_template.rs` passes Croquis and binding
  metadata to that DOM compiler. Its script-setup path derives them from
  legacy script analysis in `src/compile.rs`.
- The benchmark observations contain selection counters and output
  fingerprints, without a proof that all consumed facts are native.

## Decision

The summary reports **emitter selections**, preserving its measurements,
raw observations, timing cohorts and existing reach budgets. It explicitly
reports native-only acceptance as **not measured**. An accepted emitter,
byte-identical output, clean diagnostics or a successful process supplies
no additional native-only credit.

The aggregate also requires each of `dom_inline`, `dom_module`, `ssr` and
`vapor` exactly once in each independent runner. Four duplicate entries
cannot replace coverage of a missing backend. Invalid identities or
admission observations fail before writing the published summary.

## Validation and remaining work

Seven Node CLI tests execute the actual summary program with independent
synthetic reports. They verify accepted/diagnosed/fallback/routed/error
accounting, unchanged timing ratios, rejection of missing/duplicate/unknown
shapes, a malformed second runner, differing admission observations and
differing heads. The positive case compares the entire published Markdown
output. All seven pass locally with no skips. The repository assertion lint
scans Rust only; its successful run does not supply coverage for this
TypeScript test. No oracle exemption is added.
The original main summary fails this same full-output oracle with exit 1,
including its false native column; the corrected summary passes.

This accounting change does not establish a native-only numerator, the
corpus/dialect denominator, a stability period, or eligibility to delete
legacy code. Both tracking issues remain open. TODO: the shared differential
harness must record per-product,
per-target runtime provenance before publishing native-only acceptance
rates; missing provenance, errors and skipped execution must remain
explicitly unverified.
