# Public parser consumer packets for #10845

The adjacent `issue-10845-public-expression-packets.json.txt` retains the original
91-row observation capture byte for byte (SHA-256
`104542bf241094ee2a8d54ae32162ca16a3c648543085fdb32f9fc27a93b8576`).
It contains 90 complete expression AST/error packets for 30 controls in TS, TSX
and JSX modes, followed by the complete fatal diagnostic used for the original
#10845 input.

The final row's predecessor display label, `exact issue-10401 artifact`, is
retained unchanged. The consumer supplies
`tests/fuzz/regressions/js_ts_expression/issue-10845.input`, checks its exact
23,857 bytes and SHA-256
`60c49daaab7e06a062b0ffa7dced0eb4ddab9dbee1b52f1657fc4f1c08aa14a5`,
and compares the entire resulting diagnostic packet. Reobserving this exact input
with the local baseline and prototype produced the same complete 91-row capture.
The display label does not select input bytes.

`parser-expression-public-acceptance.yml` runs after publication with an exact
package version and immutable tag commit. It checks that the actual merged
#10945 commit is an ancestor of the tag, then creates a standalone consumer
outside the checkout with a fresh Cargo home and target. Cargo uses two build
jobs, the actual registry `vize_oxc_parser`, and registry OXC 0.142.0 allocator/span
packages. Complete Cargo metadata, lockfile, commands and outputs are preserved.

The consumer requires the original diagnostic, all 90 complete control packets,
and an original-input arena allocation below 16 MiB. The measured parser phase is
recorded as an observation. This additional public-package check complements the
source audit documented in `../ISSUE-10845.md`; the #10682 project-wide cold-time
contract remains separate.
