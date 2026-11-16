# Type-aware project batching corpus

This fixture preserves the original 121-file type-aware workload used to investigate
#10358: 40 Vue components, 80 TypeScript modules, and one shared composable. Three
templates reproduce every source byte; `authority.json` pins all 121 original hashes.
The benchmark creates ten unchanged cohorts, yielding 1,210 authored files.

`cohort.stdout` is the complete public 0.557.0 JSON output from the original cohort,
also reproduced by public 0.558.0. Only its relative file prefixes are extended for
the ten cohorts. The expanded historical reference is 1,704,822 bytes, contains 2,820
diagnostics, and has SHA-256
`bf3a6089ca4026480ebcf5b188ae11f1e757c6067f59eea0fcb3ca121bcc50dc`. It remains
immutable drift evidence, not a claim that later base revisions retain that old rule
set. The output deliberately has no trailing newline.

The frozen request vectors were observed with TypeScript 7.0.2. The former
single-document publication path has 1,211 `updateSnapshot` transports; the optimized
project path has two. That optimized vector remains a reference. Every measured shard
must instead produce a complete, non-truncated request vector with unique identities,
and all ten identical cohorts must agree within each source revision. Base and head
vectors may differ so a change that intentionally adds a type query can still be
measured rather than being rejected by stale telemetry authority. Explicit `other`
attribution is retained in that complete vector instead of being silently filtered.

The optional `patina_project_batch` input in `benchmark.yml` reuses that workflow's
exact base/head `ci-opt` binaries. It installs the locked Vue 3.5.44 consumer outside
the timed region and gives each deterministic 121-file cohort its own physical project
root. Projects share only the locked dependency installation in their parent, so checker
discovery cannot expand one shard into the other 1,089 authored files. Every sample runs
the ten shards sequentially in the same order, first as a warmup per side and then as
five alternating base/head pairs. GNU time records wall time, user and system CPU, and
peak RSS for every shard. Sample wall and CPU are sums; sample peak RSS is the maximum
single-shard resident set, never a sum. One separate complete profile sample per side is
outside the latency estimator. Input membership, source bytes, installed dependency
bytes, compiler binaries, and checker binary are checked again after the workload.
Each validated shard is also appended immediately to `shard-observations.jsonl`, so a
later shard failure cannot discard the completed shard's packet and resource receipt.
Transport spans include queueing and wire/upstream work, and do not report exclusive
worker CPU time.

Dispatch against a branch containing both the batch implementation and this harness:

```sh
gh workflow run benchmark.yml --ref <branch> \
  -f base_sha=<full-before-sha> -f head_sha=<full-head-sha> \
  -f patina_project_batch=true
```

The exact base warmup seals every complete current packet after first proving the fixed
1,210-file membership and order. Every later base and head sample must match those raw
bytes, lengths, SHA-256 digests and diagnostic counts. The historical packet is reported
separately as drift evidence. Failed output and resource observations are retained. This
fixture does not itself establish a latency result; that requires a successful
measurement for the exact source pair.
