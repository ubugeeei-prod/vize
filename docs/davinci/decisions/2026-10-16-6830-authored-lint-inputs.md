# Share authored lint inputs with the CI selector

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The fresh PR run [38905491176](https://github.com/ubugeeei-prod/vize/actions/runs/38905491176)
tested merge `a2d9dd9133b0c25b93ffbeb805bd2f97e4b3fe14`, whose main parent
`678ea68ca34ad88206670ed00749c6448cdf1b9c` introduced the virtual-list example
in #9254. The opinionated source check reported seven warnings in that example:
three static ID/reference findings, two handler-style findings and two inline
style findings. The required aggregate correctly rejected the failed app job.
Rust tests and Clippy passed; the 4m36s overall interval was a failed run, not
another successful qualification result.

The app selector covered compiler and builder changes but omitted authored
sources such as `npm/compose/core/examples/virtual-list.vue`, even though the
lint command checks them. Share one pure TypeScript input predicate between
the actual command and its dependency selector. Keep the existing source roots,
extensions and explicit fixture exclusions. A change to that predicate selects
its consumers conservatively. Markdown prose remains outside this lint input
set; ordinary package source and tests remain inside it.

Rename the lint command to TypeScript in a move-only commit, then execute it
with the configured Node runtime. The complete enumerated input list is
byte-identical before and after extraction: 4,533 authored source files. No
warning allowance or source exclusion changes.

The example now uses per-instance Vue IDs, explicit arrow handlers and scoped
CSS with the same container/row dimensions. Its virtual-list bindings, search,
scroll actions and accessibility relationships remain present.

Five existing planner tests pass on Node 24, including the newly exercised
example, package source, theme component and HTML input paths, with prose and
intentional fixtures still excluded. Scoped formatting and lint pass. The
current-source CLI and example behavior are checked by the next Actions run;
the dependency-graph fix alone is not a claim that main is fully qualified.

Integration with #9260 retains its equivalent example fix and actual event
planner control. Its narrower selector expectation for package TypeScript is
updated to the command's real input set: both composable implementations and
package scripts are already linted. Prose and intentional fixture exclusions
remain unchanged. The combined planner and package-group suites pass all 13
tests, including #9261's production Vue fixture installation contract.
