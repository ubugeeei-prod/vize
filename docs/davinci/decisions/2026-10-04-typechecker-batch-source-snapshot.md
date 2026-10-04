# Batch typechecker source snapshots (2026-10-04)

Tracking: [#7698](https://github.com/ubugeeei-prod/vize/issues/7698).
Prerequisite: [immutable dependency module facts](./2026-10-04-typechecker-snapshot-performance.md).

## Decision

Share one `TypeSourceSnapshot` through the existing private
`RuntimePropResolveCache` for each `VirtualProject::register_paths` call.
The cache already lives on that call's stack and is borrowed by its parallel
workers. It is dropped after registration, so a later scan captures dependency
edits and missing-file resolutions again. Single-file registration keeps its
fresh local cache. Concurrent projects never share a snapshot.

The SFC Croquis merge always constructs and installs a `ResolvedTypeWorld`
before resolving props. Since scoped props were selected in `389fd87223`,
its compatibility collector, normal-script flat-name collection, context
analysis and fallback props branch do not contribute to the selected result.
The world builder consumes only the current context source, supplied normal
script, syntax and snapshot. Remove that unused work within the private merge;
leave both public imported-type collectors and their respective traversal
policies unchanged. The normal/setup root script is parsed from its current
descriptor for every caller, including TSX. Dependency facts are unresolved
until each world's own clone resolves its imports.

The world's existing 512-module admission policy remains in the world
builder, before cache lookup. A snapshot warmed by smaller worlds must not
allow a larger world to admit dependencies beyond that policy. This change
does not switch between the disk compatibility collector's unbounded traversal
and the snapshot compatibility collector's bounded traversal.

## Required evidence

The source Actions test batch and single-file virtual output and diagnostics,
split normal/setup blocks, TSX syntax, a shared dependency edit across fresh
registrations, concurrent projects, and warmed-cache admission beyond 512
modules. Existing scoped-world and source-snapshot laws remain required.

The dependent PR also builds the actual CLI at its exact common ancestor and
head, deriving the full Stack baseline from its common ancestor with `main`
so the parent's first-use module cloning costs are included. Both executables
run against the same generated 500-SFC corpus and a
shared imported-barrel corpus. Each side/thread mode must retain full
normalized diagnostic fingerprints on every run and pass the existing
minimal and corpus-scale planted diagnostic gates before timing is published.
Parity preserves authored file and diagnostic order, alongside ownership,
multiplicity, clean coverage and effective program membership. Source membership
ordering alone is normalized. Each planted gate also compares full base/head
fingerprints, and a reversed multi-diagnostic vector must fail the self-test.
First-launch samples and warmed fresh-process samples are distinct; filesystem
caches are not forcibly evicted. Two untimed warmups precede nine alternating
pairs in max and one-thread modes. Profiling uses a separate process after all
timed samples. Executable hashes, exact SHAs, runtime versions, corpus bytes,
raw samples and diagnostic records accompany the report.

The historical full-command max target is 42.55 ms, derived from the 425.5 ms
baseline in [the prerequisite record](./2026-10-04-typechecker-snapshot-performance.md).
A gain in the imported graph alone does not establish that target
or native Davinci product completion. #7698 remains open until the pinned
full-command 10x criterion, real-project and persistent-session edit evidence
are met. Publication and merge are pending until exact-head Actions and the
protected native Stack queue actually finish.

## Measured Stack result and hold

[Actions run 37171536424](https://github.com/ubugeeei-prod/vize/actions/runs/37171536424)
passed the public type-world probe and actual CLI parity at exact head
`5d0c63550d62ca3287766c2819b4f7f411009dbf` against full Stack base
`da66dc241cb7e6ad25fc52fbb4a1b2c1effec9e5`. The
[paired receipt](./evidence/2026-10-04-typechecker-batch-source-snapshot.json)
preserves exact inputs, executable hashes, all nine pairs, diagnostic
fingerprints and uncertainty. The run's `type-snapshot-cli-bench` artifact
also holds every raw JSON report, input, command and separate profile.
Both sides used `ci-opt` on the same 32-vCPU Blacksmith runner, Corsa 7.0.2
and Vue 3.6.0-beta.10. Generated500 contains exactly 2,050,350 Vue bytes;
the shared-barrel fixture contains 285,350 Vue bytes and 35 TS sources.

| Corpus / mode          | First launch base / head (ms) | Warmed median base / head (ms) |     Head change |
| ---------------------- | ----------------------------: | -----------------------------: | --------------: |
| generated500 / max     |                 437.5 / 449.6 |                  433.7 / 435.0 |          +0.29% |
| generated500 / 1T      |               1344.9 / 1335.4 |                1340.2 / 1337.6 |          -0.20% |
| shared-barrel500 / max |                 416.9 / 365.8 |                  348.6 / 356.8 |          +2.36% |
| shared-barrel500 / 1T  |                 804.3 / 449.6 |                  792.3 / 459.7 | -41.98% (1.72x) |

Every first-launch, warmup, timed and profiling process retained complete
normalized diagnostics, clean file coverage and effective program membership:
500 generated files and 535 shared-barrel files. Both thread modes and both
executables caught all script, template-prop, template-event, component-prop
and corpus-scale planted errors before timing publication. The job checks
parity and provenance; green does not certify a speed improvement.

The measured 5d harness sorted file and diagnostic vectors in its normalized
fingerprint; its planted gates independently required exact authored vectors.
The strengthened harness now retains both orders directly. An
[offline ordered replay](./evidence/2026-10-04-typechecker-batch-source-snapshot-ordered-parity.json)
of all 144 saved 5d process reports passed: every normal first launch, warmup,
timed and profiling report matched its row, and all 20 planted base/head pairs
matched complete ordered reports. The receipt binds the saved raw bytes by
SHA-256. This recheck adds ordering evidence to the existing measurements;
it is not a new timed run of the latest helper revision.

Nine paired head-minus-base deltas give approximate two-sided 95% t intervals
(8 degrees of freedom; conditional on this run, not a cross-run guarantee):
generated max -1.62 ms [-9.17, +5.94], generated 1T +5.19 ms
[-13.67, +24.06], shared max +7.48 ms [-2.17, +17.13], and shared 1T
-335.08 ms [-342.36, -327.80]. Shared max was slower in seven of nine pairs;
its possible regression is unresolved and no max improvement is demonstrated.
All nine shared 1T pairs improved. A single first launch per side does not
establish a repeatable cold-filesystem gain; caches were not evicted.

Separate profiles had zero dropped spans or counters. On shared 1T, 500
`canon.croquis.augment_type_props` calls fell from 433.46 to 105.51 ms total,
and 500 `canon.vue.virtual_ts` calls from 481.97 to 149.76 ms. Its one backend
command remained 226.96 to 220.68 ms. On shared max, augmentation increased
from 1116.86 to 1478.04 ms and virtual TS from 1386.75 to 1652.23 ms.
Those max values sum overlapping worker spans and are not command latency.
They motivate investigating snapshot initialization or resolution contention;
this profile does not identify the cause. On generated max, the sharded
backend span was 330.79 to 322.48 ms, dominating the approximately 435 ms
command; on generated 1T its backend command was 1142.28 to 1160.36 ms.
Expr-parser and I/O counters matched between sides. There is no dependency
parse or canonicalization counter, so none is inferred from those counters.

The separate historical seven-run/two-warmup
[Check Benchmark Gate 37171584188](https://github.com/ubugeeei-prod/vize/actions/runs/37171584188)
passed its diagnostic gate at the same head and measured 429.5 ms max.
Comparison with the earlier 425.5 ms baseline is across runs; the same-run
paired ratios above are the performance evidence. The 42.55 ms target is
unmet. Hold native Stack #7720 (#7702 -> #7719) outside the queue while a
dependent resolver-contention slice seeks a reproducible default max gain.
Real-project and persistent-session edit acceptance remain unfinished.

## Rejected occurrence-cache experiment

[PR #7705](https://github.com/ubugeeei-prod/vize/pull/7705) was closed unqueued
after its local whole-CLI experiment showed no demonstrated benefit. The
[complete experiment record](./2026-10-04-check-import-occurrence-cache.md)
retains its paired samples, binary hashes and actual profile. The
[issue receipt](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-5975755104)
records exact base `da66dc241cb7e6ad25fc52fbb4a1b2c1effec9e5` and candidate
`a2f8356f860ffbb5c822c3001dfbe784d24dca0e`, identically configured `ci-opt`
builds in the same physical tree, Corsa 7.0.2 and Vue 3.6.0-beta.10.
Fifty alternating fresh-process pairs retained complete 502-file JSON parity
in all 100 processes. The median was 787.954 to 789.758 ms (+0.23%);
paired mean base-minus-candidate was -2.274 ms, standard deviation 54.185 ms
and standard error 7.663 ms. A separate profile observed 502 scans and 502
reuses, which does not demonstrate an end-to-end gain. This local corpus is
separate from the pinned 500-SFC Actions acceptance workload.

The binary hashes were
`b052a9ed3fb5ff62d1e119a7a3f5a31154318add939aa5b6c287b869e4ac7385` (base)
and `a80ea53c9bff3f3582828a8d960cb33d0d388c1ee18e91f27308a9455ffb649d`
(candidate). The rejected cache code is not part of this batch slice.
