# CSS engine containment references

This finite corpus keeps the exact three authored CSS inputs associated with
#3295 and the retained #4961 reproducer. It qualifies a bounded formatter and
linter containment change. It does not close the upstream retirement criteria,
upgrade dependencies, qualify an installed release, or establish native width
ownership.

`App.vue.txt` places the three unchanged originals in separate style blocks.
`App.expected.txt` independently applies the existing raw-trim SFC fallback,
standard LF block boundaries, and blank separators. The three individual SFC
references follow the same existing policy. Healthy and invalid-CSS controls
retain complete unchanged source-baseline outputs over three passes. The mixed
SFC reference composes an original failed style with a healthy later style.

The linter controls enable exactly one rule at a time. CSS parser defects retain
the existing invalid-CSS empty result for that style, while a later healthy style
still reports its complete finding. With only `a11y/no-redundant-roles` enabled,
the markerless-list route independently reaches the CSS parser. Its defect
reference keeps the existing invalid-CSS fallback: the list role is reported as
redundant. Healthy markerless CSS still preserves the role. Role-removal edits
and complete fixed SFCs are authored directly against the original source bytes.

`cases.json` contains 12 whole SFC vectors, six standalone style vectors, and
15 whole linter vectors. Rust API results include every diagnostic field, label,
and fix. `nativeExpected` retains the full public service shape for later hosted
CLI verification; it is reference data, not a claim of changed-source execution.

`source-baseline/` preserves complete original captures. The finite probes used
Node 24.14.0 in isolated child processes and the known source-built H339 binary
with SHA256 `b884696038e3e6c09054b73d9532a3ab490d2cd296a5770d836561b0be245d1d`.
They confirm SIGABRT for each original formatter, CSS-only lint, and independently
enabled markerless-list route. Healthy and ordinary invalid-CSS controls exit
successfully. The binary came from source commit
`3390cb6044d53d655a9d64e112d2618375cff5ca`; the recorded ten relevant source files
and four locked dependency blocks match main
`b41811e3b33676f68a0843a693c9ed4898f2e86a` exactly. These captures establish the
unchanged source baseline only. Fresh hosted execution must qualify the fix.

`issue-3295.body.txt` and `issue-4961.body.txt` preserve the complete live issue
bodies read during preparation. The separate #4961 CSS source is the retained
reproducer, which is not quoted in that issue body. `custody.json` hashes every
fixture file other than itself and identifies both observer source files.
