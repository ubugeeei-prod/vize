# Rust compilation inventory for protected Check

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).
Status: measured inventory and [CLI/SFC-sharing PR #8617](https://github.com/ubugeeei-prod/vize/pull/8617)
under Actions qualification. No complete source qualification, measured saving,
or merge is claimed for the prototype.

## Source and observations

The inspected main commit is
`78ac30f2f97b6f6ec681affdded5ad66cc44e61b`. Both original protected Check
runs finished successfully, with 30 successful and 17 deliberately skipped
jobs each. Their complete 47-job inventories remain the check inventory;
scheduled/manual Nix and coverage jobs were skipped, not qualified here.

- [38127019057](https://github.com/ubugeeei-prod/vize/actions/runs/38127019057)
  checked `f440b7d7c1d39bec78268f45570d6499780a3e13`.
- [38127454609](https://github.com/ubugeeei-prod/vize/actions/runs/38127454609)
  checked `78ac30f2f97b6f6ec681affdded5ad66cc44e61b`.

This inspection reads committed recipes, complete existing job logs, and
Actions job/step metadata. It starts no build, test campaign, or workflow run.
The timings below distinguish Cargo's printed command duration from the
whole Actions step, which also includes fetching, receipt writing, and copying.
They do not measure CPU time or isolate an optimization's effect.

## Repeated CLI and oracle builds

The [apps job](https://github.com/ubugeeei-prod/vize/blob/78ac30f2f97b6f6ec681affdded5ad66cc44e61b/.github/workflows/check.yml#L210)
and each of the four [tooling jobs](https://github.com/ubugeeei-prod/vize/blob/78ac30f2f97b6f6ec681affdded5ad66cc44e61b/.github/workflows/pr-source-checks.yml#L127)
run `cargo build --profile ci -p vize` on separate Blacksmith workers.
Tooling also builds `vize_atelier_sfc`'s `cli_build_result_oracle` example.

| Run 38127454609 job     | CLI Cargo duration | Oracle Cargo duration | Complete CLI step |
| ----------------------- | -----------------: | --------------------: | ----------------: |
| Apps, 114434674608      |            37.29 s |             Not built |              46 s |
| Tooling 1, 114434787763 |            34.99 s |                0.30 s |              36 s |
| Tooling 2, 114434787714 |            37.74 s |               17.43 s |              55 s |
| Tooling 3, 114434787691 |            40.18 s |               17.92 s |              58 s |
| Tooling 4, 114434787769 |            38.48 s |               18.36 s |              57 s |

Tooling 1 had already built native SSR targets in the same `ci` target tree.
This explains a different warm dependency state; it is not an excluded test.
In the preceding protected run, apps' CLI Cargo command took 39.04 s and
tooling 3's CLI/oracle commands took 39.22 s and 17.41 s. All four tooling
CLI steps took 36–57 s. The repetition is present in both successful runs.

The four current tooling receipts name the same checked source and recipe,
but contain different actual executable digests. Independently produced
binaries are not asserted to be byte-identical or reproducible. A shared
consumer must authenticate the producer's actual bytes, not manufacture its
own receipt or borrow an old job's digest.

## Existing sharing and cache boundaries

The [Rust producer and four shards](https://github.com/ubugeeei-prod/vize/blob/78ac30f2f97b6f6ec681affdded5ad66cc44e61b/.github/workflows/pr-rust-checks.yml)
already compile workspace tests once into a nextest archive. Each shard
verifies its source/toolchain/runtime receipt and runs its original partition
from that archive. Adding another test prebuild would duplicate an existing
mechanism. The protected producer's archive step took 120 s and 110 s in
the two runs; Clippy took 28 s and 27 s, and doctests took 43 s and 20 s.
Default-profile Clippy, `ci` tests/doctests, and feature-enabled differential
corpora have distinct commands and feature/profile contracts.

Every inspected Rust-cache invocation in the two protected runs reported
Actions cache, a registry miss, a 184,128,522-byte Cargo git hit, and a target
miss. Sticky mount steps explicitly skipped. The
[policy](https://github.com/ubugeeei-prod/vize/blob/78ac30f2f97b6f6ec681affdded5ad66cc44e61b/.github/actions/setup-rust-sticky-cache/cache-policy.mjs)
only writes/mounts trusted exact-main push, scheduled, or manual checkouts;
PR and merge-group consumers remain restore-only. Distinct physical runner
names provide no job affinity or shared mounted target directory.

Sticky caching does work for trusted jobs. Original main job
[114409178710](https://github.com/ubugeeei-prod/vize/actions/runs/38118691486/job/114409178710)
mounted the three sticky paths and hit the existing stable apps target seed,
with a 2 s cache post step. That job's success is separate from the whole
superseded run. A trusted seed hit does not prove a protected consumer hit.
The current consumer misses' backend cause is not established by these logs.
Keep the trust boundary and the existing 25 GB oversized-target refusal.

PR-to-queue repetition is also visible in
[38125924287](https://github.com/ubugeeei-prod/vize/actions/runs/38125924287).
Its API head is `ec303c5c87a8fdb73e4136eaffcab14d1fd39fdc`, while its actual
checkout and CLI receipt name PR merge object
`673897e77a384ee5fb71d73e0ec2761e31d4b59f`. It built CLI and affected Rust
tests before the protected full source ran again. The queue adds another
PR's actual Patina source and uses full workspace/required-TSGO coverage;
the PR archive uses affected packages and disabled TSGO. These artifacts
cannot substitute for the protected source/runtime envelope.

Native-addon builds are a separate opportunity: current tooling 2–4's
debug `napi,legacy` builds took 44.93–46.44 s; JS's debug and later `ci`
native builds took 44.86 s and 39.73 s. Their
[formatter-history capture](https://github.com/ubugeeei-prod/vize/blob/78ac30f2f97b6f6ec681affdded5ad66cc44e61b/npm/native/scripts/formatter-history-build.mjs)
retains the complete Cargo process, emitted artifact, profile, source, and
toolchain, and preparation requires a live local owner. Keep those contracts;
neither a `ci`/release addon nor a fabricated debug receipt satisfies them.

## Narrow producer and consumer change

Build the ordinary `ci` CLI and SFC oracle once in a small reusable producer
inside PR source checks, after the existing source plan. Only tooling needs
that producer. JS, Rust, canonical corpus, playground, and WIT retain their
existing dependencies; root apps and standalone native workflows stay intact.
Every tooling setup, native capture, original test partition, and required
report remains. The final source report also requires the producer to succeed.
An unrelated selection runs its explicit successful skip explanation.

| Boundary                               | Existing dependency             | Prototype dependency                 |
| -------------------------------------- | ------------------------------- | ------------------------------------ |
| CLI/SFC producer                       | Four independent tooling builds | Source plan                          |
| Tooling 1–4 initialization             | Source plan                     | Source plan and CLI/SFC producer     |
| JS, Rust, canonical corpus, playground | Source plan                     | Source plan                          |
| Root apps, standalone native           | Existing independent workflow   | Same                                 |
| Source report                          | All seven original gates        | All seven gates and CLI/SFC producer |

This explicitly delays tooling initialization until the narrow producer is
available. Native SSR/DOM/Vapor hooks nested inside tooling inherit that delay;
their commands and controls remain. There is no polling, dependency on the
whole apps job, or dependency on the later workspace-test archive.

The producer runs both original Cargo commands and creates the original CLI
receipt. Its complete Actions build log remains the build authority. Existing
artifact actions carry a four-member tar: CLI, unchanged receipt, SFC oracle,
and transport identity. Consumers verify the producer-emitted archive digest,
exact run/attempt, actual checked SHA/tree, platform, Rust/Cargo/Clang versions,
actual Wild bytes/version, complete Cargo-home configuration, environment flags,
both recipes, paths, modes, and complete member hashes before installing.
The pinned Wild action stores its linker under each job's `RUNNER_TEMP` and
uses that path in Cargo-home configuration. Only that owned linker path is
normalized for cross-job comparison; other configuration bytes remain exact.
Missing/corrupt members, links, foreign bytes, and identity mismatches fail.
Consumers copy the genuine receipt byte-for-byte without restamping it.
SFC observations retain the authenticated producer identity and verify the
oracle's full bytes before and after its original process invocation.

The source report's ordered gates use a flow sequence to keep the original
350-line limit. Existing workflow laws follow the genuine producer/restore
dependency and retain every original workload and refusal assertion.

## Expected cost and unfinished measurement

The four CLI and oracle Cargo durations above sum to 205.40 s of command wall
time across parallel workers. A single approximately 38 s CLI plus 18 s oracle
would leave approximately 149 s of repeated aggregate compile wall removable,
before adding packing and four consumers' transfer/validation cost. This is a
projection from existing logs, not CPU time or a measured after result.

A narrow producer may finish 65–75 s after the source plan using the observed
setup/build costs. Current tooling 2–4 reaches its CLI build 46–52 s after that
plan, then spends 55–58 s building it; shard 1's SSR-warmed build takes 36 s.
Delaying initialization and replacing those builds could place the boundary
roughly 7–20 s later for shards 2–4 and 29–39 s later for shard 1, before
measured transfer costs. These estimates require actual hosted comparison, including queue
availability and the final longest job; they do not prove a global speedup.

Do not wait for the workspace archive producer: its job finished at
09:15:50, after all four tooling CLI boundaries. Do not broaden sticky trust,
drop profile/feature checks, or adopt Bazel/Ninja from this evidence. Cargo
already supplies the needed compilation/archive mechanisms.

## Initial hosted qualification

[Check 38133096382](https://github.com/ubugeeei-prod/vize/actions/runs/38133096382)
checks original head `5c6513b0bf2198b7f95f80cb33095c7aa725adbf` using actual PR
merge source `48526ca27f944f706974bc055aff7127439ad036`. Its shared producer
and all four strict restorations succeeded. Cargo printed 36.57 s for CLI
and 17.62 s for oracle; the complete build step took 63 s, packing 7 s,
uploading 1 s, and each restore 4–5 s. The 53,180,406-byte artifact's complete
four-member content and modes were independently checked without executing it.
The actual producer became available 89 s after the plan, and tooling began
90–91 s after it. This is slower than the earlier 65–75 s availability estimate;
the initialization delay remains a cost to assess in the complete queue run.

That source remains red: the original evidence-upload law still required
consumer-local receipt creation, and the source-preflight law still required
tooling's former single dependency. Their full original job logs are retained
as 888,794 bytes / SHA256 `ff1acc4e62bedd563aad7661fee5314724840d589dc9110a4f0156f1ce8f151d`
and 679,834 bytes / SHA256 `e0e0581bc717d0b26be838b6aab993e3aef94278289981f290cfca236898b3ef`.
The bounded correction follows the genuine producer, strict restoration,
original receipt and final always-upload path, and preserves every failed-plan
refusal with tooling's exact two dependencies. Product and transport bytes
remain unchanged. Fresh corrected-head Actions must qualify all original gates;
these initial successful boundaries provide no credit to that later source.

## Retired protected candidate

[Check 38135879923](https://github.com/ubugeeei-prod/vize/actions/runs/38135879923)
at `872957a02d953bf4afe42f5c3e708f288ce43763` passed all original full source
workloads: 31 jobs succeeded and 17 retained their existing skips. Its CLI
and oracle commands took 54.36 s and 17.54 s; the complete build took 81 s,
packing 6 s, and the four strict restorations 4/4/5/4 s. The producer became
available 110 s after planning; tooling initialized 117 s after it. JS tests
took 906 s and UI 140 s. These step walls do not prove CPU or global savings.

The candidate remains red and was dequeued: its original
[Musea run](https://github.com/ubugeeei-prod/vize/actions/runs/38135879860)
passed 46 of 47 cases but timed out waiting for the first current variant.
Both diagnostic callbacks failed with `__name is not defined`, leaving the
actual navigation state unknown. The [browser payload correction](./2026-10-11-6830-musea-browser-payload.md)
preserves the original wait and corpus; it repairs collection, not the timeout.

TODO: qualify final composed source and all original job/workload outcomes.
Compare aggregate compile command wall removed, packing/transfer/validation
added, and overall critical path independently. The earlier baseline JS job
took 1,216 s in both protected runs and was the longest job. Removing repeated
Rust commands could reduce resource use without shortening that observed
whole Check path; no overall speedup is claimed before measurement.
