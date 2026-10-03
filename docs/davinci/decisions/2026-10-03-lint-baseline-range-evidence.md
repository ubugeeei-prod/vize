# Invalid lint baseline range evidence

Refs [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

## Decision

Retain each rejected ESLint finding in the optional
`divergence.baselineInvalidRanges` array, with its normalized project-relative
file and complete original message object. Include that array in the existing
evidence hash. Add the field only when an invalid range exists, preserving the
shape and hash of valid evidence. Apply the same rule to the canonical Rust
Script reporter and retained JavaScript compatibility reporter.

The observed real-project run discarded original invalid findings before
uploading artifacts. Its aggregate counts cannot establish which rule or
coordinates caused the failure. Retaining the complete payload makes a fresh
failure diagnosable without inventing replacement coordinates.

## Validation and limits

The focused TypeScript suite verifies original nested payload retention after
caller mutation, valid findings surviving beside invalid ones, differing
coordinates producing differing hashes with identical counts, and unchanged
valid-evidence hashes from the actual pre-change reporter. Existing invalid
range evidence still produces an unusable budget verdict. The two existing Rust
collector laws also assert full rejected payload retention; real Rust execution
and complete Actions validation remain required.

The new Rust Script execution law belongs to the existing full T1 tooling
inventory. Pure comparator and budget laws remain in T0; no job or gate is added.

This change preserves range validation, classification, zero divergence budgets,
parser errors and gate behavior. It does not fix invalid coordinates or establish
real-project compatibility. TODO: capture original findings on a fresh pinned
real-project run, diagnose their source, fix the responsible behavior, and rerun
the mandatory ecosystem gate. The explicit expected-zero Vue-file repair is
tracked separately in PR #7495.

## Finite hosted capture

Dispatch the existing Real Project Matrix with
`capture_invalid_lint_ranges: true` on the literal reviewed capture source.
Its default remains false. This selects one lint-only job and skips both the
22-project tool matrix and S2 corpus for that capture invocation. Normal and
scheduled matrix commands and the short PR/full protected Check remain intact.
Capture and full-matrix concurrency groups are separate.

The job verifies ten original producer/control file pins, all nineteen original
fixture contracts and gitlinks, the unchanged frozen whole repository lock,
Node 24.14.0 and installed ESLint/parser versions. It hydrates only those
fixtures, builds the commit CLI once and refuses the reporter Cargo fallback.
It invokes the canonical Rust reporter once with the literal nineteen-project
CSV, ecosystem preset, coverage-gap measurement, one shard, original 600000 ms
timeout and enforce mode. No coordinates, rule selections, expected-file
counts or zero budgets are relaxed.

The reporter original stdout, stderr and process status are preserved. Complete
report verification rejects missing projects, changed identities/budgets and
finding/count disagreements. Partial output remains explicitly partial. All
observed reports, Markdown summaries, process records and preflight data upload
after failures; lossless gzip/Base64 log frames bind every original byte and
retain the failed process status. The final step returns the original reporter
exit. A transport or verifier failure also fails the workflow.

Local finite laws verify original nested payloads, fresh counts differing from
the historical 224, source/revision/budget/count refusal, incomplete/duplicate
summaries, exit propagation and frame truncation/corruption/foreign-stream
refusal. No fixture hydration, package installation, CLI build, ESLint producer
or hosted capture has run from this source. TODO: Root publishes the reviewed
source and runs this single capture, then diagnoses the original findings and
validates a genuine correction through the mandatory ecosystem gate.
