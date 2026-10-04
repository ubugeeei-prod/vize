# Complete Rust evidence after partial workflow reruns

Issue: [#7773](https://github.com/ubugeeei-prod/vize/issues/7773).
Paired [decision comment](https://github.com/ubugeeei-prod/vize/issues/7773#issuecomment-5977742840).
Initial source base: actual main `9702c014e06a268d2651202b7fcb951db7e729df`.

## Observed failure and authority

Protected Check37182670251, candidate
`7cebaf4aadf87586f2970fb35a9aef0c251d5d65`, remains FAILED. Attempt1 worker1
was cancelled during archive download without running tests. Its attempt2 rerun
downloaded the same archive and passed 3,921 tests. Peers2–4 retained attempt1
successes. The original archive-download delay remains unexplained.

The source report selected only attempt2 artifacts. Pinned download-artifact
3e5f45b2 flattens one artifact even without merge-multiple. The collector then
treated `junit.xml` as a directory and failed with ENOTDIR. Independent audit of
the four small official ZIPs confirms shard1@2 plus shards2–4@1 retain identical
archive receipt bytes and executable identity, 14 mandatory Canon bodies,
31 complete original project vectors and 15,486 successful JUnit cases.
The 3.64GB archive was not downloaded by that data audit. It does not grant
workflow success or qualify any other candidate.

## Bounded change and gates

The existing report downloads immutable worker artifacts from the current run
across attempts. Trusted runner run ID/current attempt reach the collector
explicitly. Canonical directory names must identify shards1–4 of that exact run
and positive attempts no later than the current attempt. Missing, malformed,
flat, duplicate, foreign-run and future identities refuse before packet reads.
For each shard, select its highest attempt before validating packet contents.
An invalid newer packet fails; no older successful packet can replace it.
The packet shard must agree with its selected artifact name, and the log keeps
the actual selected attempt identities rather than relabelling inherited peers.

The unconditional current-needs Rust gate stays first and required. Failed,
cancelled, skipped, missing or unresolved required builder/shard results still
fail the source report regardless of earlier artifacts. Existing full execution,
source/tree/manifest identity, required TSGO/disabled-null runtime, toolchain,
common receipt/archive, common actual executable, successful JUnit, every
registered body and every complete vector/input/public-result check remain.
No new API permission, worker schema, runtime flag, pipeline stage or budget is
introduced. Prior report/acceptance files are cleared before validation, and
new output is written only after all four selected packets pass.

## Qualification and remaining work

Focused laws cover partial/full reruns, artifact/context refusals, current
dependency failures and invalid newer source/tree/manifest/archive/runtime/
executable/JUnit/shard custody without fallback. Twenty-one targeted local tests
pass without skips. These synthetic tests provide accounting/refusal proof;
fresh exact-head Actions and protected full/100+4 instruction gates remain
required before actual signed merge and final attribution verification.

Reporter `ubugeeei`, GitHub ID71201308, is the verified primary account.
Source/PR retain its public noreply Co-author; final same-primary metadata
normalization must be reported as observed. Native/default/history migration,
measured10x and the original download cause remain unfinished. Root's finite
v0.432.0 batch excludes this PR; queue admission waits for verified publication.
