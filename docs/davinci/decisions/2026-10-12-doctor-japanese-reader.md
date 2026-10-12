# Doctor Japanese reader flow

Refs: #8365 (More guides) and #8372 (natural Japanese).

## Decision

Author the complete Japanese Doctor guide against the current practical English
page, leaving that whole English source unchanged. Preserve all four full shell
examples, every inline command and filter token, all category/severity/confidence
values, every filter dimension, and all original English heading aliases.
The reviewed-translation marker retains the whole authored regeneration input.

Explain the first report, source inspection, JSON output and actual next steps
in natural Japanese. Preserve the diagnostic-only boundary: no findings means
only that the current analyzers found no issue in the selected sources. Filters
select after analysis, retain the evidence graph, and reconstruct normal scoring
and blocking state; --path and --changed-file keep their distinct scope.

Source authority is the CLI doctor arguments/filters/output and exit policy,
vize_doctor filter.rs and filter/matcher.rs, DoctorReport reconstruction and
summary blocking policy. Repeat-within-dimension OR, between-dimension AND,
case-sensitive globs, normalized slash paths, failure status 2, default certain/
high-confidence error blocking and process-only --exit-zero remain unchanged.
No analyzer, CLI, provider, report or installed-release behavior is changed.

## Delivery boundary

Update only the finite /guide/doctor More entry. Native English/Japanese reading,
whole examples and source fragments, actual next-step links and stable published
source identity are required before claiming this bounded guide delivered.
The complete More and Japanese editorial issues remain open.

Prepared in a fresh isolated worktree on actual 0e0839 while the official casing
publisher and ready performance/lint native Stack continued. After both readers
actually merged as a4c80f and 563eb3, replay the unchanged approved guide onto
actual ba7774, including the integrated native pointer/selection examples.
Append only this companion link to the existing foundation row 337; all incoming
clauses and all 350 physical rows remain exact. Casing helpers, renderer/capture
sources and other owners' configuration preservation fixtures remain unchanged.
Fresh source Actions and actual merged-source reading/publication are required;
do not dispatch a redundant whole-site or unrelated native benchmark campaign.
