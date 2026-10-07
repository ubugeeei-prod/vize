# CLI autofix convergence

Issue: [#7906](https://github.com/ubugeeei-prod/vize/issues/7906).

The CLI currently applies one round of nonoverlapping edits. The reported
`v-bind:title="title"` binding needs a second invocation to remove its value
because the two fixes overlap. The native API already repeats up to ten passes.

Repeat the CLI's existing edit selection and source-specific lint entry until
there is no edit, the source stops changing, or ten passes have run. Re-lint after
each changed source so returned diagnostics and counts describe the final bytes.
Keep the original file untouched during all passes and atomically write only the
final source once. Write failures keep the existing error and exit behavior.

Retain at most eleven source states to detect contradictory rule cycles. A cycle
returns the original source and original diagnostics without writing the file;
repeated invocations therefore cannot alternate the authored file bytes. A
noncyclic sequence that exceeds the limit retains the tenth pass and its
remaining diagnostics, matching the existing native API's finite limit.

The immutable reporter body, original TitleBox and authored final TitleBox live
in `tests/_fixtures/differential/linter/lint-fix-passes-7906/`. Rust laws cover the
original overlap, a second stable invocation, disabled fixes, absent edits,
no-op edits, cycles and the ten-pass bound. The actual CLI regression checks the
fixed file, final empty plain diagnostics and a second stable invocation.

Formatting is checked locally. Exact-head Actions, protected queue checks,
actual merge and next release remain pending. The other missing edits in #7905
are separate work; no completion is inferred from this preparation.
