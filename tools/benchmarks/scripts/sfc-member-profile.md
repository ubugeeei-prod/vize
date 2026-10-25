# Native SFC prop-member phases

Dispatch the existing Criterion workflow with `sfc_member_only=true`, an immutable main-ancestor
baseline, and the exact candidate SHA selected by `--ref`:

```sh
gh workflow run criterion-bench.yml --ref BRANCH \
  -f base_sha=BASE_SHA -f head_sha=HEAD_SHA -f sfc_member_only=true
```

The mode is exclusive. Every existing job and ordinary ceiling keeps its default behavior when
this flag is false. This report never replaces normal source CI, correctness or instruction gates.

Both N-API images are built before measurement using the same optimized `ci-opt` recipe
(`opt-level=3`, thin LTO, 16 codegen units). The actual loaded image must match its build token and
SHA-256. Every side uses an initially absent, role/SHA-specific target directory in a fresh runner
temporary namespace. Compiled targets are never restored or shared, including when the two source
SHAs are identical; only downloaded Cargo registry/git sources are reused. Source-tree IDs,
target directories, dependencies/lockfiles, immutable source SHAs, driver/input hashes and build logs are
retained. A driver-only control must first report `productionIdentical: true`; production tuning
then compares against the same baseline with the same frozen corpus.

The matrix contains 32, 128 and 512 bare ASCII, readonly ASCII, quoted, readonly quoted, Unicode
and readonly Unicode members. Keyword boundaries, comments/whitespace, cooked literal names and
refused dynamic expressions are correctness controls. Complete `profile: false` JSON packets
must be byte-equal across every side/repetition. The profiler envelope has a closed schema and is
kept separately; no metadata fields are normalized away to manufacture equality.

Thirty source-cache-bypassed samples follow three CPU warmups per shape. Thirty cache-hit samples
follow an explicit primer. Runs use base/head/head/base order in isolated Node processes. This
measures source-analysis demand separately from process startup, file IO and result-cache hits.
Raw phase vectors and actual work counts are retained, together with p50/p95. Timing differences
are observations, with no relative threshold or release-throughput claim. A source optimization
must also prove its intended parse-count reduction in normal correctness tests.
A production-identical run only qualifies the driver. An optimization must include an actual
changed-counter negative control so accidentally loading the baseline image cannot pass as its head.
