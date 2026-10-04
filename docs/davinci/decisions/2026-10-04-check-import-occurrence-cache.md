# Rejected lexical import scan retention experiment

Issue: [#7698](https://github.com/ubugeeei-prod/vize/issues/7698).
Decision: [issue comment](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-5975590068).

Default `vize check` preparation walks the complete source graph before and
after adding hidden ambient declarations. The existing session already shares
package lookup and registration facts, but each walk reads and scans every
source again. Referenced default programs repeat this preparation independently.

The experiment kept a source occurrence cache in those repeated-walk sessions. Each access
still reads the complete current file. Only identical source bytes reuse the
previous lexical scan; an edit replaces it, and a failed read evicts it. Cached
occurrences retain their individual contextual, import or require mode and the
existing lexical treatment of comments, raw SFCs and incomplete syntax. Import
targets are resolved again, so creating a previously missing relative target
does not require changing its importer. The cache adds no parser or pipeline
stage and does not change diagnostic scope or ambient registration rules.

Source retention is enabled only for known repeated default-program walks.
One-pass explicit scopes keep the existing owned occurrence vector and do not
retain source bytes or copy occurrences into a cache. Retained sources and
occurrences live only as long as the preparation session. This trades bounded
session memory for repeated scanning; it does not reduce disk reads.

The regression suite covers same-session same-length edits with restored mtime,
missing-target creation, failed reads and recreation, occurrence-mode and
lexical parity, and 500 SFC roots sharing one module. The latter two-walk case
must execute 501 scans and reuse 501 scans, rather than scanning 1,002 times.
`--profile-json` exports `check.import.scan.calls` and `check.import.scan.reused`.

The fail-closed Check Benchmark Gate's pinned invocation selects an explicit
dot path. It checks regression safety for that path and does not measure the
default repeated-walk benefit. Acceptance needs exact-head Actions and separate
default-invocation measurements. This preparation slice does not establish the
10x full-command goal; that criterion in #7698 remains open.

## Result: reject and close without merging

[Result decision](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-5975755104).
[Raw paired receipt](./evidence/2026-10-04-check-import-occurrence-cache.json)
and [actual profile](./evidence/2026-10-04-check-import-occurrence-profile.json).

Fifty alternating paired default checks compare exact main `da66dc241c` to
source candidate `a2f8356f86`, built with the same `ci-opt` profile, Corsa 7.0.2
and Vue 3.6.0-beta.10. Every complete JSON report matches: 502 files and the
same planted TS2322 diagnostic. Two warmups precede each variant's timings.
Median fresh-process time is 787.954 ms before and 789.758 ms after (+0.23%).
The mean paired before-minus-after difference is -2.274 ms, with 54.185 ms
sample deviation and 7.663 ms standard error. A shorter 15-pair run pointed
in the opposite direction. These local runs show no whole-command benefit.

The actual profile confirms 502 scans and 502 reused scans. Eliminated work
alone does not justify retained bytes and another cache. Corrected source
head `a8dcd9920c` passes Check and all planted checks in
[Check Benchmark Gate](https://github.com/ubugeeei-prod/vize/actions/runs/37170349514).
That separate explicit-path gate is regression evidence, not paired default
performance evidence. PR #7705 is closed without entering the queue; no
source is adopted and no release is required.

The retained residual is 2,015,553 source bytes generating 11,275,428 virtual
TypeScript bytes (5.594x). A 62-byte shared TS dependency connects all 501
SFC roots, collapsing seven requested checker shards into one. Its backend
profile is about 554 ms. A follow-up may investigate a bounded cost model
for cheap shared script leaves while preserving target resolution, ambient
registration, ownership, ordering and complete diagnostic deduplication.
The full-command 10x criterion in #7698 remains open.
