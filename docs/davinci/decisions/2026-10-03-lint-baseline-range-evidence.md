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
