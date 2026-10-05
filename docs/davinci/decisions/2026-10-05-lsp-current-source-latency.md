# Bounded current-source LSP latency and memory baseline

Decision for [#3952](https://github.com/ubugeeei-prod/vize/issues/3952).
This is measurement tooling for the shipped LSP. Actual measurements, GUI
latency, native/default/history admission and the typecheck 10x target remain
unfinished. Source/pure checks never substitute for timed hosted execution.

## Fixed workload and source custody

One manually dispatched Actions invocation runs three serial fresh server
processes. Each uses only the existing pinned Misskey frontend pair and original
two phases of 20 four-edit cycles in `lsp-churn-stress.test.ts`, retaining its
complete diagnostic, cancellation, lifecycle, order, RSS and latency gates.
Ordinary callers keep the original workload. The baseline adds one completion
after each clean cycle, outside the existing cycle timer: 120 cycles, 480 cycle
edits and 120 extra completion RPCs across the three sessions. This explicit
extension is not a byte-identical historical replay or a broad project matrix.

The exact reviewed dispatch SHA must equal checkout HEAD and `GITHUB_SHA`.
The build is literally `cargo build --profile ci -p vize`, followed immediately
by the existing source build receipt. Every server launch requires that exact
binary, source receipt, executable hash and actual version probe; fallback is
refused. Source/tree/config/lock identities and the allowed runtime environment
are retained before execution and checked after each session. No full secret
environment or credentials are collected. Actual job/run/attempt identifiers,
Node/Rust/Cargo/Python versions and hardware accompany the results.

The pinned fixture gitlink and full selected frontend file manifest are retained
before spawn. The isolated copy must match every pinned original input. All
four authored unsaved source variants remain complete text plus hashes, and raw
RPCs keep their exact inputs. The explicit real backend path/version/hash is
recorded before execution and must match an observed descendant executable.
Original includes/settings, symbols and fixture bytes are unchanged.

## Timing and memory meaning

Complete client/server/stderr bytes and chunk offsets retain all message fields,
arrays, nulls, omissions and order. Node monotonic clocks index the chunk that
completes each actual frame, including split UTF-8 and coalesced frames. Client
timestamps precede bounded observer writes and the existing stdin write; they
are not server acknowledgements. Synchronous raw writes and the independent
sampler are explicit instrumentation overhead, not an uninstrumented estimate.

Only unique-version leaf edits enter primary broken/repaired distributions.
Each must match the complete original consumed diagnostics within its wire
window. Shared dependency edits republish an unchanged leaf version: compatible
whole vectors and post-send ordinals are preserved separately, with causal
application explicitly unproven. The original four-edit cycle timings remain
available. Completions retain request ID, current URI/version/content hash,
authored position and complete response, using the original required-symbol
oracle and existing registry ceiling. No full ranking oracle is invented.

Warm lanes report all samples, nearest-rank empirical p50/p95 and maxima, per
session and for the same workload aggregate. Spawn-to-initialize-ready and first
leaf diagnostic observations retain all three values and their median; startup
p95 is null because three processes do not establish a tail. Launch resolution,
source hashing and the existing version probe are timed separately. Fresh
processes use inherited runner/OS/build caches and do not imply cold caches.

A separate Linux `/proc` sampler targets 50 ms, recording actual clocks,
sampling gaps, PID/start-time, observed executable hashes, every live observed
descendant and reparented known children. Reused root PIDs and foreign members
are refused. Sampled maxima and RSS sums are neither true peaks nor PSS;
short-lived processes between samples remain unobserved. Shutdown requires a
complete footer and no live observed descendants; forced cleanup retains failure.
The original server/tree/process ceilings and budget scale1 stay strict.

Raw wire and test logs each have a 16 MiB session bound; RSS has a 64 MiB bound
and 330 s deadline. Original tests retain 300 s and the controller has a 350 s
cleanup deadline. Missing/truncated/stale evidence, failed or empty completions,
failed original oracles, caps or cleanup refuse acceptance. All partial raw
evidence and failures remain. A failed/missing/extra session prevents aggregate
statistics instead of selecting a favorable subset; unknown timeout totals are
null. A refused stale retry removes the earlier public assessment, retaining raw
evidence. The manual workflow uploads partial evidence even on failure.

## Current limits and next real-RPC regressions

Authenticated historical scheduled Check `37192966110` at source `caee165692`
had Misskey incremental/churn and Vben incremental artifacts. Misskey's 40
four-edit cycles had nearest-rank p50 299.176 ms / p95 310.737 ms; those are
historical cycle observations, not current single-edit or completion p95.
Protected main Check `37198344364` at `61c975f888` had no LSP performance
artifact. No historical green or measurements transfer to this new source.
The new driver has not run a real server yet; hosted baseline dispatch requires
source peer review and separate coordination. Full #3952 remains open.

P0 [#7835](https://github.com/ubugeeei-prod/vize/issues/7835) next needs its
original complete strict-bundler project: bare and compound template uses of
every `void`-substring identifier, plus unaffected controls. Real hover ranges,
complete references and whole rename edits must be compared from declaration
and template; applying every edit must prove all authored uses changed without
touching surrounding text. Generated-source projection custody is a prerequisite.

[#7824](https://github.com/ubugeeei-prod/vize/issues/7824) next needs its original
complete Child/Parent project in a fresh server where only Child is opened.
Manifest/config custody must prove the unopened Parent is included. Complete
references with declaration and whole rename edits must cover both files;
an excluded outside-project file must stay untouched. Opening Parent is a
separate positive control, not a workaround credited as a fix. Other source
owners coordinate these production corrections; this baseline changes neither.

Editor TS-45 initialization correction #7836 and Nuxt2/3/Nuxt4 build proofs are
separate correctness lanes. They cannot establish current Rust/Vapor/SSR memory
or latency, native migration or actual GUI responsiveness. Optimization follows
the finite measured baseline; no fast, instant or 10x result is claimed here.

The maintainer's delegated review accepted the finite three-session campaign
after an independent review of all sixteen changed paths. Its fresh-main replay
retains all fifteen noncanonical source blobs from `34a6f19e8f` exactly and adds
only its existing decision clause to the complete `d1a25ec1da` record. Fresh
head Actions and protected merging precede one dispatch on the resulting
literal main SHA; historical source checks do not qualify that execution.
