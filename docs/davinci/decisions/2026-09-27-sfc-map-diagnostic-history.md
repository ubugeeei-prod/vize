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

TODO: commit the observer source cleanly, make a narrow locked profile-ci build
on this agent's exclusive existing target, bind the selected executable to its
source SHA, capture/repeat actual output, then freeze it without normalizing or
editing golden bytes. Populated map and diagnostic evidence, regression test,
strict Clippy and fresh Actions are not yet measured. Whole-history coverage
and native compiler acceptance remain unfinished; do not close #6880.
