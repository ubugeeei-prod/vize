# SFC map and diagnostic history references

Tracks [#6880](https://github.com/ubugeeei-prod/vize/issues/6880). This next
small cohort preserves two original authored inputs from genuine fixes:
#3604 `0494c409b` and #750 `7b821f8d0`.

The Counter source and options come directly from the original
`compile_sfc_attaches_a_map_only_when_the_flag_is_on` test: descriptor/compile
filename and script ID `/app/src/Counter.vue`, inline_template=false, Standard
syntax and codegen source_map=true. The historical test pins map presence,
sources/content and an authored segment, but not the entire public Result.
The later structured-map feature #6365 replaced that implementation. Capture
the actual current implementation; do not call it the original fix's output
or count that feature as another historical fix.

The props default-type mismatch input and compiler options come directly from
`test_props_destructure_default_type_mismatch_errors`: parse defaults and
compile defaults with script ID `test.vue`. Its original code/message checks
remain. The new observation retains the entire public compile Result including
location. That entrypoint currently reports script-block-relative locations.
Also observe the public located semantic validator with the parsed setup
content, its actual content start and the full original source. This additional
route reports document-relative spans; retain both Results and their distinct
contexts without conflating them or claiming an LSP runtime acceptance.

The test-only observer reads actual options and serializes complete public
Results, keeping all generated strings, map scalars, diagnostics, bindings,
macro artifacts, Result variants and array order. It uses `.input.txt` sources
outside the global authored Vue corpus, changes no product behavior/dependency,
and keeps every existing live-lane, segment and editor witness.

## Actual complete output capture

Tracked-clean source `72293b11bc2a4b3d0f3671626af0c67bf5ebfec5` was built with
`cargo build --locked --profile ci -p vize_atelier_sfc --example
sfc_map_diagnostic_fixture_observer --message-format=json-render-diagnostics`
on this agent's existing exclusive target. The actual narrow build succeeded
in 1.95 seconds. Its selected example artifact was copied off target and hashed:
`41d2cc09f6277366b7073a018deb88d852ddc39444a45e1932e415314c778098`.

The receipt binds committed clean source/tree, toolchain, lock hash, argv,
selected Cargo artifact, successful process and raw build-log hashes. Complete
raw stdout/stderr/process records from two actual executions are retained and
repeat byte-equally. No reference comes from an intended build, stale target
binary, substring, Insta normalization or source projection. An earlier failed
compile of preparation source `c70e63a2d` is retained externally and receives no
execution credit.

Counter's complete Ok has a populated map from the current structured map
implementation. The count mismatch's complete compiler Err includes all eight
location fields with byte span 17..23; the additional located-validator Err
includes all eight fields with document byte span 41..47. Both keep the full
message/code and separate contexts. Both report line 2/columns 17..23 here,
but their offsets belong to different source coordinate systems. The captured
values, not these summaries, are the expectations.

The frozen reviewed source remains reachable at
[`provenance/sfc-map-diagnostic-history-capture-72293b11b`](https://github.com/ubugeeei-prod/vize/tree/provenance/sfc-map-diagnostic-history-capture-72293b11b).
Publication preserves every indexed archival byte and regenerates only the
bounded SFC consumer shard for the new helpers. This provenance reference
preserves historical local measurement identity; current-head Actions are
a separate prerequisite.

The ordinary integration test compares the entire observation, including
options, both Result variants, map values, diagnostics, bindings, macro
artifacts, arrays and every generated string byte. Exact case count/order guards
prevent missing cases. The fifteen-file hash index preserves source inputs,
provenance, raw build/process captures and convenient per-case views. JSON
object-key order alone is immaterial; all string bytes, scalar/null variants
and array order remain exact. Existing authored witnesses stay unchanged.

Observer/test wiring uses ordinary module discovery. The observer entry moves
in a move-only commit; small test-only helper copies have identical logic after
module and fixture-path adjustments. This avoids new dependency crates or
path-attribute bypasses. Strict Clippy passes the final example/test with warnings
denied, and the complete-result test passes after those wiring/lint changes.
Captured stdout and archival Cargo JSONL/receipts are never reformatted or
regenerated to satisfy lint. The original capture source is historical local
measurement identity, separate from fresh final-source execution.

This is two inputs and two genuine fixes, with three public Result observations.
Across the existing compiler cohorts, that prepares 23 inputs linked to 15
concrete fixes, without pretending that the issue's 609-fix snapshot denominator
has been reconciled. Fresh Actions, parent main/full queue proof, whole-history
coverage and native compiler acceptance remain TODO. Do not close #6880.

## Existing stack merge acceptance

CI4 reached actual main `42014675678684165b8cd4bb992bb9db747e74ee`.
Its tree matches the frozen CI4 source, and every preceding CI merge SHA is
an ancestor. The sole inspector owner published #6955 at
`96dff355b431a1ee321d9616d0eeaf4b09aed1fb` on that main. The approved
compiler suffix is transported coherently behind it, keeping the inspector
record under Toolchain practice and all compiler pointers under L4: emission.
Each member preserves the actual-main text and its complete owned prefix.

The existing group is #6955 → #6930 → #6933 → #6937 → #6954. New PR
publication remains frozen. The four compiler cohorts retain all 91 fixed
fixture/capture files and their original source/executable/archive receipts.
The reviewed compiler crate bytes and provenance refs remain unchanged;
transport does not turn historical measurements into current-head execution.

Fresh completed Actions on every exact new head are required before queueing.
Require generated successful Check/source reports, the Rust executor and all
four test shards, every required check, source caps, archive exclusions and
bounded consumer inventories. An empty pending/failure list is insufficient.
The [earlier SSR Check](https://github.com/ubugeeei-prod/vize/actions/runs/36308996672)
failed first on the inspector pipe race, then on the failed-only retry's
archive mismatch; it remains a historical failure.

After every latest head passes, link the existing PR numbers with
`gh stack link --base main 6955 6930 6933 6937 6954`, mark eligible drafts
ready and run `gh stack merge <stack> --yes --squash`. The official command
uses the native merge queue. Verify every real full queue prefix and the
historical test execution, then all five actual merged states and main merge
SHAs. Enqueueing or an individual successful PR Check is not final merge proof.

The published compiler cohorts contain 23 authored inputs and 15 distinct
genuine fix links. The private extra two-case capture is not counted. The
609-fix snapshot denominator is unreconciled, whole history and native
acceptance remain unfinished, and #6880 stays open.
