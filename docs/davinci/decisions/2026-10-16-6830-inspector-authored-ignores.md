# Respect authored ignores in the independent Inspector census

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The source-built App qualification at `4a3fe42686fc9713b0fafaccb47684a5622ead13`
retained 71 original VueFes inputs in both Inspector reports, but the independent
coverage helper counted 122 files after genuine Nuxt preparation. VueFes pin
`8eff3cd7dd79a7f47bbf2b78202b9aa2070ec5a6` generates 51 additional Vue components
through `modules/01.mdc-prerenderer.ts`. Its authored `.gitignore` line 32 excludes
`app/components/_i18n`. All 71 original source byte hashes match the pinned
archive; the preparation receipt preserves all 593 authored files unchanged.
The extra files are regular generated files, not missing original coverage.

The native Inspector's `input_files.rs` uses an ignore-aware walk for directory
inputs and simple `root/**/*.vue` patterns. Existing explicit files and generic
glob patterns use separate branches that bypass ignore filtering. The helper
now preserves those distinctions and asks Git to filter only the walk's
candidates. No application-specific exclusion is added.

[`git check-ignore --no-index --stdin -z`](https://git-scm.com/docs/git-check-ignore)
evaluates ignore rules independently of tracked status and exchanges NUL-delimited
paths. Exit 0 means some candidates are ignored; exit 1 means none are ignored.
Other exits and process errors fail the coverage check. There is no fallback to
an unfiltered or smaller guessed inventory. This helper serves the current Git
fixtures: it does not claim complete equivalence with `ignore::WalkBuilder` for
standalone non-Git trees, `.ignore` rules, hidden-file overrides or symlinks.

The original count and sorted-path SHA-256 assertions remain unchanged, as does
the exact report-file comparison including duplicate detection. An unignored
new source, removal or same-count substitution still fails. Source pins,
compiler options, directional budgets and original fixture bytes are unchanged.

Five Node 24 tests cover overlapping selection, omitted/duplicate/substituted
reports, wrong summary counts, empty inputs, physical source addition/removal/
rename, 51 generated ignored files, nested ignore negation, an ignored file
force-added to the Git index, directory/recursive selection, explicit ignored
file inclusion, generic-glob inclusion, fatal Git exit and missing Git. Replaying
both retained hosted VueFes reports against the genuinely prepared local tree
passes the unchanged 71-file path identity
`ab1162e3967372fa4f651c1cd39d4ceb37b85de84d79769ff908f6a8935a8c72`.
This validates the coverage helper against retained evidence; full combined-source
App qualification remains an Actions check.
