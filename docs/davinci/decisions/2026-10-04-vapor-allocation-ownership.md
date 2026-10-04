# Vapor allocation-window ownership diagnostic

Issue: [#7764](https://github.com/ubugeeei-prod/vize/issues/7764).
The naming slice [#7757](https://github.com/ubugeeei-prod/vize/pull/7757)
remains draft and outside the merge queue pending a real diagnosis or fix.

Protected candidate `e1c0c8609e64035a9705cd76257e9d3fdaf966b4` failed
[Check 37181078166, shard 2](https://github.com/ubugeeei-prod/vize/actions/runs/37181078166/job/111375266501):
`text_runs` native 78 / retained 76, with the unchanged native ceiling 75.
All seven measurements are retained in the
[original failure receipt](https://github.com/ubugeeei-prod/vize/issues/6832#issuecomment-5977207839).
Accepted parent `b9be9065b872086726158b67d1b4e7b8dee78a14` passed the
same case. Its successful stdout was not stored; passing is no numerical
baseline. Candidate and parent have identical Vapor production, allocation
counter, fixture, Cargo.lock and nextest-config sources. The L1 feature command
ran after the full archive had already been stamped, listed and uploaded.

Rust 1.98 libtest spawns the test worker, then its main thread can allocate
the running-test map, timeout deque and result-channel bookkeeping.
Thread-count 1 still spawns a worker and waits on its channel. Source inspection
establishes a possible overlap mechanism; it does not attribute the observed
three extra calls. Process-global `CountingAllocator` totals include genuine
workers spawned by the measured routine. Thread-local totals, increased
ceilings, arbitrary extra warmup and rerunning until a passing count appears
would change or evade that contract.

The maintainer authorized one isolated diagnostic branch, issue and finite
Actions experiment. No production fix or #7757 source change follows from
this authorization. The counter/test delta and complete recipe require
independent source review before dispatch.

## Observer contract

The explicit `davinci_harness/allocation-window-diagnostics` feature adds an
allocation-free fixed atomic ledger. Every successful alloc/zeroed/realloc
keeps its original process-global call/live/peak updates and receives a global
ordinal, Linux kernel TID, size and operation label. Destructor-free constant
TLS caches positive gettid results; invalid identities reject the receipt.
Fresh process BSS supplies the ledger before warmup. No array clear, heap work,
stack unwinder, new pipeline stage or serialization occurs in a measured
window. Endpoints are the existing global counter snapshots; storing one fixed
window slot takes constant time after the original metric snapshots.

A combined closing-bit/writer-count CAS gate prevents an accepted late event
reservation after snapshot closure. Pending writers, incomplete publication,
overflow or invalid identities are fatal. Every exact measured allocation
ordinal must join once. Completeness is scoped to those exact measured
ordinals; the ledger makes no exhaustive whole-process-history claim.
Instrumentation can change scheduling and executable bytes, so instrumented
observations remain distinct from the official original executable controls.

All seven source literals, ceilings, warmup and native/retained order remain.
The observer flushes after all fourteen windows and seven stdout rows, before
the original ceiling assertion. An over-cap attempt must keep exit 101 and
the unchanged failure message. A positive control performs actual direct
counted operations and spawns a real counted routine worker; the worker's
allocation remains in the global total and in its separate TID label.

## Finite Actions recipe

The existing Check workflow accepts an explicit boolean input. Its diagnostic
job runs only on `ci/vapor-allocation-ownership-20261004` under manual dispatch.
Source PR checks and ordinary full workflow gates keep their existing policy.
The diagnostic job uses the pinned Linux runner, Rust 1.98, nextest 0.9.146,
wild 0.9.0, CI profile and required TSGO/Nuxt environment.

The experiment executes exactly 64 fresh processes: two frozen sources ×
original/instrumented executable × thread counts 4/1 × eight attempts.
Every original exit, stdout, stderr and diagnostic trace is preserved. There
are no retries, count selection or best-of result. Validator synthetic laws
exercise missing/duplicate windows and ordinals, invalid identities, incomplete
publication, overflow, stdout joins and original cap-failure propagation;
they grant no real allocation or causal execution credit.

Original controls are the complete official full-workspace archives:

| Source     | Run         | Archive artifact | Complete ZIP SHA-256                                               |
| ---------- | ----------- | ---------------- | ------------------------------------------------------------------ |
| `b9be9065` | 37180545481 | 11294942508      | `c9a43b2722809f24966f81814e0e470f94c8689801212ef7792ff0f467a48539` |
| `e1c0c860` | 37181078166 | 11294949814      | `5dc11633be55cb47b38472524efb2305d5e721486bbf40a54fd6b23357e268f0` |

Official API metadata, complete ZIP digest and stamped archive digest must
agree. Authenticated owning timing artifacts provide all Cargo compiler-unit
feature inputs and the 15,519-case inventory. The actual budget ELF path and
digest are retained alongside full archived Cargo/nextest metadata. Large ZIPs
are processed sequentially, then deleted after authentication; only selected
ELFs and metadata remain. No archive byte-equality claim is made between sources.

Each instrumented arm overlays only the reviewed counter/test feature files
onto its frozen source and builds a cold full-workspace nextest archive.
Every compiler-unit feature tuple must match the original envelope except for
the explicit harness diagnostic feature. Locked JS build manifests must match
before sharing installed dependencies. Instrumented ELF/archive hashes, overlay
hashes, exact command/environment and actual feature arrays remain separate.
Exact original `rustc -Vv`, pinned wild config and source profile/config/lock
inputs are enforced; ambient rustflags, encoded flags, target-directory,
compiler-wrapper, profile and target overrides are rejected. Selected ELF
dependencies are checked and recorded before launch. Final completeness also
requires the exact 64-key source/mode/thread/attempt cross-product and one
immutable executable identity for every source/mode arm.

The independent checker requires contexts 0 through 13 exactly once in order,
non-overlapping counter endpoints, complete ordinal joins and equality with
every printed native/retained count. PID identifies the libtest main thread;
measuring-worker and other-worker allocations remain counted. The exact
per-event size/operation/TID witnesses are retained, including both cap successes
and failures. Thread-count 1 is diagnostic contrast and grants no fix credit.

## Deferred acceptance

Until actual authenticated execution demonstrates an over-cap contribution,
the observed +3 ownership remains unknown. A complete matrix with no
reproduction must report that limitation and retain the original failure.
Missing arms or incomplete trace data fail the experiment. No diagnostic status
grants protected acceptance or makes #7757 eligible for the queue.

After actual ownership evidence, the maintainer owns the genuine fix decision.
An existing standalone custom-harness precedent preserves process-global
counting and genuine spawned workers, but it is not adopted by this diagnostic.
The final fix will need exact source Actions, unchanged performance caps,
protected candidate acceptance and actual merge evidence.

The first diagnostic dispatch on `1e54247cf1` (Check 37184897212,
job 111384678751) stopped in explicit-feature Clippy: a compile-time Linux
assertion triggered `assertions_on_constants`. The matrix step was skipped;
none of the 64 planned processes executed. The existing positive actual Linux
TID assertion now carries that platform contract without a redundant constant
assertion. A preparation-only identity receipt is written before source checks
so setup failures remain uploadable. This repair preserves all ordinal/identity
guards and measurement semantics; it grants no cause or acceptance credit.

The successor `7727cf948b` passed explicit-feature Clippy, then stopped at
matrix launcher initialization (Check 37185278742, job 111385802809): the new
preparation step had already created the directory. Artifact 11296996677,
complete ZIP SHA-256 `8b02e561d76dd7f7664acee03c8c7a819d2095aabc9bf911606b58e87b82b5db`,
authenticates only a preparation receipt with zero attempts and false credit.
No archive or measured process was consumed. Initialization now admits only
that exact fresh head/run/attempt preparation, refuses any previous matrix
evidence, and preserves initialization errors without overwriting existing
records. A direct initialization law checks the actual preparation-to-runner
integration plus wrong identities and reused-output rejection. Matrix inputs,
allocation hooks, windows and caps remain unchanged; cause remains unknown.

## Completed bounded campaign: cause unknown

Paired [actual outcome and pin decision](https://github.com/ubugeeei-prod/vize/issues/7764#issuecomment-5977908002).

Producer `fdfe97336b238d6078e207f5fe54ec64f054d7c0` passed source
[Check 37185712103](https://github.com/ubugeeei-prod/vize/actions/runs/37185712103).
The authorized finite campaign's
[diagnostic job 111387387331](https://github.com/ubugeeei-prod/vize/actions/runs/37185823768/job/111387387331)
succeeded. The overall manual workflow also succeeded; ordinary manual full
validation is separate from campaign completeness and establishes neither
protected merge-group acceptance nor the historical allocation cause.
Official artifact `11296109680`, named
`allocation-ownership-37185823768-1-fdfe97336b238d6078e207f5fe54ec64f054d7c0`,
has complete ZIP SHA-256
`53118497a544521d102da77b2fad34ddf9861e17930fa1085c562b5087e40e13`.
Independent audits authenticated official metadata, every retained file hash,
all 64 unique source/mode/thread/attempt keys, all seven printed rows and
original exits. There are 32 original archived ELF attempts and 32 distinct
instrumented ELF attempts; every attempt exited zero without timeout.

All 64 native/retained vectors were identical:

| Fixture      | Native | Retained |
| ------------ | -----: | -------: |
| text_runs    |     74 |       76 |
| events       |     72 |       72 |
| expressions  |    106 |      133 |
| components   |     91 |      117 |
| templates    |    132 |      143 |
| spreads      |    104 |      122 |
| control_flow |    155 |      174 |

The 32 instrumented attempts contain 448 exact measured windows and 50,272
joined global allocation ordinals. Every measured call belongs to its measuring
worker; no libtest-main or other-worker call occurred inside these windows.
Both genuine positive controls retain one actual spawned worker allocation
alongside the measuring allocation in the global total. Each source's 1,536
compiler-unit feature tuples match its original envelope apart from the
explicit observer feature; rustc identity, linker/config, ELF dependencies,
source inputs and overlay hashes are authenticated.

| Source     | Original budget ELF SHA-256                                        | Instrumented budget ELF SHA-256                                    |
| ---------- | ------------------------------------------------------------------ | ------------------------------------------------------------------ |
| `b9be9065` | `41b80ec59763a5dc4e2bb8d1973e782adaf237500e2ccb4e848811e47b793f00` | `2b1ff951a8c8f5e99e21530c41df7b516b6b77bb50ef2de0cd1661bd3881edcc` |
| `e1c0c860` | `9d6b0cad11a699c1b59d9e89a4cca8e7dc24e53c1e2434691d50098b38a9c956` | `43c5590eeeb100139f8be272602fe83745d335013d2ec2e9de50b97917a71bb5` |

The original archives share selected-suite metadata and executable paths, but
their actual budget ELF bytes differ. Fresh native 74 does not attribute either
three calls above ceiling 75 or the difference from the historical 78. There
was no over-cap reproduction. Instrumentation may change scheduling, direct
launch is a diagnostic control, and thread-count 1 remains diagnostic contrast.
The campaign launched its processes serially and directly. The historical
full nextest shard ran 3,826 cases across 916 binaries; its profile permits
16 concurrent test processes with inherited `RUST_TEST_THREADS=4`, while the
actual simultaneous count at failure was not recorded. Protocol reconstruction
from pinned official nextest 0.9.146 source
[`8af696ddcce8fff2962d6a5168b6d138b8616a35`](https://github.com/nextest-rs/nextest/tree/8af696ddcce8fff2962d6a5168b6d138b8616a35)
uses pipe capture, null stdin, the default Unix double-spawn stub/process group
and additional Cargo/NEXTEST runtime environment. The diagnostic instead uses
regular output files, inherited stdin, direct spawning, forced backtrace 1 and
source-specific relocated ELF/worktree paths. The old actual execve, complete
child environment, descriptors and stub use were not captured; reconstructed
protocol is not a historical runtime trace. The quiet campaign therefore does
not replay the full historical driver or rule out its scheduling effects.

The original failure, both zero-attempt setup failures and every finite outcome
remain retained. Cause remains unknown; no production fix, ceiling increase or
ratchet, protected acceptance, additional trial or #7757 readmission follows.
The diagnostic PR remains draft and outside the release batch and queue.

A separate code-scanning check on producer `fdfe97336b` failed at the
Rust-toolchain action pin, although the source Check workflow succeeded.
The inherited `6bed0761` is an official generated stable-branch commit outside
master's history. The upstream
[full-SHA pin policy](https://github.com/dtolnay/rust-toolchain/blob/7e38f4b43b4db5c8dd498af069a4f6196df1d067/README.md#choice-of-full-length-commit-sha)
requires master history, and its
[branch generator](https://github.com/dtolnay/rust-toolchain/blob/7e38f4b43b4db5c8dd498af069a4f6196df1d067/scripts/update-revs.sh)
replaces generated toolchain branches. A narrow successor pins only this new
action to verified official master `7e38f4b43b4db5c8dd498af069a4f6196df1d067`
while retaining explicit Rust 1.98.0. Fresh source checks must validate that
successor; producer `fdfe97336b`, its action bytes and campaign provenance stay
historical. Counter, fixtures, ceilings and matrix scripts remain unchanged.
No new 64-process campaign is authorized by this pin correction.
