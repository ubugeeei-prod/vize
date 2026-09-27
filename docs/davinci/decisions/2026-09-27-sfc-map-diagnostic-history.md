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
