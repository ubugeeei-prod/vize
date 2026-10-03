# Expected zero-Vue lint surfaces

Tracked in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

Real Project Matrix [37098199031](https://github.com/ubugeeei-prod/vize/actions/runs/37098199031)
rejected four fixture projects because they selected no Vue files:
`vue-native-core`, `docsify`, `petite-vue` and `wakapi`. Their existing registry
records explicitly expect zero Vue files. The input collector already honored
that expectation, but the later lint-budget usability check discarded it and
rejected every empty corpus. These failures do not establish a Vue 2 compiler
regression; other matrix diagnostic, parser and range failures remain separate.

The Rust reporter and its JavaScript compatibility implementation retain the
registry expectation as `files.expectedCount` next to `files.comparedCount`.
Only numeric zero authorizes an empty corpus. Missing or positive expectations
still reject empty selection, and an expected-zero project with any selected
Vue file fails before either linter runs. Existing registry entries, rule maps,
presets, workflow enforcement and false-positive/false-negative budgets remain
unchanged.

Both actual ESLint entry points use `passOnNoPatterns` only for the supplied
empty list. ESLint otherwise treats `lintFiles([])` as a request to lint the
current directory, which would silently compare unrelated JavaScript inputs.
This preserves the registered Vue surface; it does not exempt diagnostics.
Mapped rules must still exist, parser errors and invalid ranges remain fatal,
and all divergence budgets still apply. A complete expected-empty surface is
valid report input and provides no nonempty linter-parity evidence.

Focused laws exercise both actual reporters with a synthetic hydrated registry
and a controlled CLI diagnostic result. The positive law includes invalid
unrelated JavaScript and requires zero compared/excluded inputs, mapped rules,
unchanged zero budgets and a passing aggregate. Negative laws refuse missing
or incompatible expectations and unexpected Vue files before lint execution.
A nonempty Vue input with an actual mapped upstream finding must still breach
the unchanged zero-false-negative budget.
Budget laws retain parser, range and diagnostic failures for expected-empty
reports. The old budget rejects the new positive law; the corrected helper
passes eight focused laws. Actual Rust-runner and exact-head Actions validation
remain required before merge. No matrix-wide acceptance is claimed.
