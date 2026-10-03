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

The first bounded capture (run 37115392207, source 0a871) built the CLI but
failed runtime preflight because Git reported tracked changes after hydration,
install and build. The reporter did not run; only the original source receipt
and partial verification were framed and uploaded. Retain the original red
run. Record the exact changed paths, repository status/diff and actual fixture
HEAD/status/diffs before the same strict dirty-source refusal so a subsequent
capture can diagnose the real change. No dirty path is exempted or restored,
and no original finding or successful reporter exit is claimed.

The private first diagnostic bcb373 failed a real 1,365,125-byte Git-diff
counterexample at Node child-process default output buffering before saving
evidence. Preserve that failed proof. Stream Git stdout/stderr directly to
owned evidence files, compute complete size/SHA-256 with a fixed-memory read,
and reference large outputs in the bounded diagnostic record. Small original
values remain inline; the inline threshold never limits raw capture. The
unchanged recursive frames and artifact directory retain every raw file. An
actual greater-than-one-MiB Git-diff law checks exact complete bytes and full
frame recovery alongside the original nested-gitlink refusal law. The dirty
source refusal and actual nineteen-project capture remain mandatory.

The streamed-drift capture (run 37117688600, source c3f7454c) failed
before fixture hydration because checkout depth two omitted pinned reporter
commit ddef7f37. No reporter executed or original findings were captured;
partial verification and the actual missing-ancestor error were retained.
The direct correction requires four commits: correction, c3, 0a and ddef.
Fetch that exact ancestry without changing the pinned reporter or ancestry
assertion. A real local shallow-Git law rejects depths two and three and
accepts depth four. Preserve both original failed hosted runs. Fresh hosted
execution is still required; no invalid finding has been made usable.

The next original capture (0f7e42fe, run 37118686362) reached installed
runtime verification but refused the unchanged source guard before reporter
execution. All nineteen actual fixture HEADs matched their fixed revisions.
The other eighteen fixtures were clean; shadcn-vue reported 734 registry JSON
changes. Complete retained diffs contain only CRLF/LF comparisons. Actual
original-blob reproduction establishes that the physical CRLF bytes survive
ordinary first checkout while upstream text/eol attributes make Git classify
its own original blobs as dirty; installation-induced rewriting is not
established. Preserve the 91 complete partial log-frame files, original
failed exit and issue #6830 comment 5968741971. There are still no original
reporter findings or successful producer exit.

For this capture alone, initialize and fetch the same pinned shadcn revision
without a checkout. Before its FIRST checkout, write per-repository info
attributes only for the two existing registry JSON patterns, disabling text
and eol conversion for those paths. Keep Vue/style attributes inherited.
Compare every physical tracked file or symlink with the original full Git
blob and mode, retaining the complete tree and hashed inventory through the
unchanged always-frames and artifact path. The other eighteen fixtures use
the original shallow submodule command; all nineteen revisions, Vue globs,
producer rules and budgets are unchanged. No dirty fixture is restored,
ignored or normalized. The unchanged whole-source guard remains mandatory
after hydration, installation and CLI build.

The real Git law retains original CRLF and Vue/style bytes, reproduces the
ordinary self-dirty checkout, proves the raw FIRST checkout is clean, and
refuses a later one-byte mutation with the actual unchanged guard. Complete
blob attestation independently rejects that mutation. The unchanged frames
recover the full attestation, tree and dirty-source evidence. Capture-only
fetch-depth zero retains the original pinned reporter across the genuine
five-commit source chain; normal Matrix and PR gates are untouched. Ten
finite laws pass; actual hosted nineteen-project capture is still required.
