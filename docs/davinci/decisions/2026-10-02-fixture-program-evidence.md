# External TypeScript program members in fixture evidence

Issue: [#7376](https://github.com/ubugeeei-prod/vize/issues/7376)

The enforced real-project matrix at `6b3b0919f79407530be78da6739eb9de7a1429c0`
([run 36888121981](https://github.com/ubugeeei-prod/vize/actions/runs/36888121981))
rejected Inertia, Vue Cropper and Vue Fes Japan Speakers typechecker JSON because
`programs[].files` included Vite's declaration file from ancestor `node_modules`.
This metadata is deliberately part of the complete TypeScript program graph;
the native CLI has included `compilerOptions.types` declarations since #6695.

Keep the native JSON contract and all program members. Extract the canonical
Rust program validator in a move-only commit, then accept normalized relative,
POSIX absolute, Windows drive and UNC program-member paths. The compatibility
JavaScript oracle validates the same path forms independently of its host OS.
Empty, non-string, NUL, backslash, repeated-separator and dot/traversal forms
remain invalid. Diagnostic and requested-input paths retain their strict
relative-path contract, and external declarations do not count as checked
fixture sources or change coverage digests.

The owning `tests/tooling/fixtures/typechecker-external-program.json` fixture
reduces the Inertia output from the failing run to one checked SFC and its
ancestor Vite declaration. Both validators check the complete fixture, retain
its bytes and metadata, compare coverage against a relative-only control and
exercise malformed paths and absolute diagnostic-file rejection.

The same matrix reported authored LSP completion-count differences without
listing actual candidates. Include the complete candidate labels in the failure
message and complete hover contents in missing-hover failures to identify the
next real discrepancy; keep every expected count,
ranking oracle, divergence allowance and performance ceiling unchanged.
