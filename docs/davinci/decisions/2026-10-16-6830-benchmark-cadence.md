# Keep repeated performance studies outside the PR loop

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

Fresco dimension measurements run nightly and by explicit dispatch. Their
three repetitions still compare the frozen standard parser with the candidate
in the same process, including scalar parity and 1k/10k-node layout workloads.
Both implementations come from the recorded source commit. String Storage
runs only by explicit dispatch with full ancestor and candidate SHAs; the
candidate must match the dispatch revision. Its three paired runs retain the
identical harness, semantic equality, eight Criterion estimates, sample checks
and raw artifacts. No performance assertion or workload budget changes.

At `f48c27ac186c88ed52e9aa1d601c11c4cec1a5c4`,
[Fresco](https://github.com/ubugeeei-prod/vize/actions/runs/38869052606) passed
in 76 seconds (3.5 runner-minutes), and
[String Storage](https://github.com/ubugeeei-prod/vize/actions/runs/38869052537)
passed in 157 seconds (7.4 runner-minutes). Neither exceeded five minutes.
Removing their PR triggers avoids about 10.9 runner-minutes of repeated study
work on a change that selects both. This reduces iteration cost and duplicate
profile builds; these measurements do not establish either as the critical path.

Daily affected Rust tests still execute Fresco's non-ignored dimension parity
tests and Croquis CF semantics. The affected Fresco package conformance job
already compiles the NAPI addon to check generated declarations and consumer
types; it now also runs the strict NAPI and test-corpus Clippy commands. The
existing source report requires that job in PR and merge-queue checks.

To collect fresh measurements at a candidate ref:

```sh
gh workflow run fresco-dimension-bench.yml --ref "$candidate_ref"
gh workflow run string-storage-bench.yml --ref "$candidate_ref" \
  -f "base_sha=$base_sha" -f "head_sha=$head_sha"
```
