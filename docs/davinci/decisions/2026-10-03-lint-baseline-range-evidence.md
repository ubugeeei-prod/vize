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

The c598 capture (run 37120746827) actually executed the canonical Rust
reporter on all nineteen original projects. It retained all 61 original
files, nineteen JSON/Markdown reports, the full summary, stdout/stderr and
original producer exit one. The 224 observed invalid findings are all
Vue comment-directive controls: 108 disable-rule and 108 enable-rule
messages, plus four disable-all and four enable-all messages, all at
column zero. Every original report remains UNUSABLE; this is capture
evidence, not ecosystem acceptance. Issue #6830 comment 5968991650 records
the actual terminal failure.

The actual pinned Vue plugin wires its vue/vue processor in the flat base
configuration. The canonical embedded ESLint collector omitted that
processor, so internal controls reached the report instead of being
consumed by the provider’s own directive handling. Connect that exact
processor, preserving every rule, coordinate and budget. A meaningful
Node law extracts the actual embedded collector and executes real pinned
providers on real temporary Vue files; it verifies complete diagnostics
and enabled/disabled directive behavior without a copied collector.

Post-reporter verification separately failed its unchanged per-fixture
zero-status assertion: seventeen stdout bytes were observed, but the
fixture identity and original bytes were not retained. Their cause is
unidentified; neither a generated summary nor a cache path is established.
Before that same assertion, retain the phase, actual fixture identity and
revision, original normal status, complete status, tracked diff and
untracked names. Stream complete Git stdout/stderr into owned files with
size/hash references; unchanged always-frames and artifacts include them.
A real committed fixture law observes an exact seventeen-byte synthetic
status, refuses it, recovers every original byte through the actual frames
CLI, and preserves the first diagnostic after a later refusal. The test
filename is a fixture counterexample, not a guess about the hosted path.

One direct c598 successor combines these bounded corrections. Its current
reporter Git blob/hash is pinned, and preflight records the verified
campaign HEAD as reporterSource while retaining ddef as the explicitly
historical reporterBaseline ancestry pin. Full-history checkout is
unchanged. Original nineteen revisions, Vue globs, dependency versions,
rule mapping, source assertions and enforced budgets remain exact. Fresh
actual capture is still required; historical 224 is comparison only.

Eighteen finite laws pass on the combined source: seven real canonical
provider laws and eleven capture/retention laws. The real provider law
is registered in the existing T1 inventory; ordinary short PR selection
and the original registered capture workflow remain unchanged.

The e0 capture (37123576674) retained all nineteen complete reports with
zero invalid baseline ranges. Six budgets passed and thirteen breached:
144 false positives, 182 false negatives and 37 rule-location differences.
The enforced producer still exited one. Post-verification separately
refused seventeen status bytes for alexandrie: the exact untracked
node_modules/.vize/vize.config.schema.json path. Its HEAD and tracked
files were unchanged; the generated file contents were not captured.
Issue #6830 comment 5969413865 binds the original 180 files and terminal
failure. This does not identify c598’s earlier unretained seventeen bytes.

The canonical reporter already uses lint --no-config. The CLI nevertheless
materialized its optional editor schema before configuration loading.
Move that call into the existing configured branch: no-config lint leaves
fixture files untouched, while configured lint keeps schema generation.
Do not redirect diagnostics, ignore generated paths, restore fixtures or
weaken the original source guard. Original nineteen fixture/rule/runtime
pins, severity and every enforced budget stay unchanged.

An explicit Vue/config/full-JSON legacy regression runs the actual source
CLI binary. It compares complete diagnostics and exit one, every fixture
file byte and the Git HEAD/index/status for both missing and stale schema
cases. The configured control checks the exact bundled schema and the
same full diagnostics. The existing capture build runs only this focused
CLI test target before the original nineteen-project producer. No new
local CLI execution or hosted capture success is claimed; fresh source
validation remains required, and the thirteen real budget failures are
separate unfinished work.

The actual 9c capture (37126104242) stopped at both focused CLI laws
before the nineteen-project reporter. The fixture expectation wrongly
borrowed a diagnostic from an explicitly enabled rule; the ecosystem
preset produced the same complete JSON with no messages, errorCount zero
and warningCount zero, exit zero and empty stderr in both invocations.
Retain the failure, original 52 files and full log. Correct only that
observed JSON and expected exit; the Vue/config inputs, production fix,
workflow and nineteen pins/rules/budgets stay unchanged. Missing/stale
schema cases still require every file and Git value to remain exact, and
the configured control still requires the exact bundled schema write.
Those inventory assertions were not reached in the failed run, so fresh
actual CLI validation and the original nineteen capture remain pending.
