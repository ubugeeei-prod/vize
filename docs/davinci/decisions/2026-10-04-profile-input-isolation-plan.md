# Caller-owned profile export inputs (2026-10-04)

Proposed next bounded slice for [#6834](https://github.com/ubugeeei-prod/vize/issues/6834).
Source baseline: actual main `4158645a8858bbd2428fae08e651479758243bee`.
The preceding caller-owned timing Stack #7789 actually merged as signed
`82ed972a` then `5d4b9410`; its six plus two new and 14 preserved laws,
actual full suites and both candidates’ 104 probes × three passed.
This private plan supplies no execution, race-reproduction or new merge proof.

## Actual remaining platform boundaries

| Boundary | Actual source and caller | Remaining scope |
| --- | --- | --- |
| Clock | `profiler/core.rs` stores native `Instant` in `Timer` and nested `ProfileFrame`; native `TimingObserver` uses the same global gate and Timer. | Native timer and guard clock ownership remains host coupled. Accepted `WalkTiming<Mark>` itself reads no clock. |
| Profiler TLS | `PROFILE_STACK`, allocation suppression depth and monotone thread allocation counters live in L0. | Nested guard bookkeeping and real allocator hooks remain thread local. |
| Profiler locks/global state | Three arrays of 32 `RwLock<FxHashMap<...>>`, relaxed global enable flags, and the lazy global profiler live in L0. | Host sessions own actual native aggregation and measurement. |
| Arena TLS | `pool.rs` keeps each worker’s idle arenas and checkout counter. | Preserve reset/drop/generation and thread-teardown fallback; do not replace this pool with a lock or unmeasured mirror. |
| Recursion/platform | `ensure_sufficient_stack` directly calls the pinned `stacker::maybe_grow`; native and legacy recursive callers depend on it. | Stack growth and deep-recursion acceptance need a separate host policy review. |
| IO/path | `source_io::decode_utf8` is borrowed validation; native `read_to_string` reads the filesystem; path canonicalization calls the OS and retains Windows normalization. | Pure decoding already exists. Filesystem and canonicalization adapters remain explicit host work. |
| std/type dependencies | Allocator parking uses owned boxes/vectors; generation/cache use atomics; reports and metrics name std aliases; dependencies still include stacker and serde. | Empty default features are not no_std. Alias changes alone would not establish isolation. |

The browser build is `wasm32-unknown-unknown`. The existing scheduled/manual
stage-library lane builds `wasm32-wasip2`, including no-default-features.
Neither route establishes bare-metal support or complete no_std isolation.
This proposal changes no workflow or target lane; CI #7779 belongs to its owner.

## Smallest concrete consumer gap

`crates/vize_curator/src/inspector/stages/profile.rs::ladder_profile` accepts
only the real caller’s `LadderStep { stage, pass, nanos }` arrays. It creates a
local `Profiler::enabled()`, records only attributed durations, exports them,
then disables that profiler. Those source calls reset process allocation
counters and toggle the global allocation-tracking flag; the collector also
constructs the 96 sharded locks. Its public comment consequently restricts
concurrent native profile sessions. No native timestamp is needed here.
This is a source-observed boundary, not an executed race diagnosis.

The existing L0 `Metrics::record(Duration)` already performs the needed
count/min/max/self/child/histogram arithmetic without a clock, TLS or a lock.
Carton’s `export_report(&Profiler, &ProfileExportOptions)` consumes only the
existing owned span and counter readbacks. Reuse these actual APIs and wire
records; add no clock trait, general profiler framework or new metric model.

## Exact telemetry contract before any consumer change

Curator currently starts an empty local collector, calls only
`record_attributed`, and passes `allocation: None`. Its real output therefore
has top-level `allocation: null`, every span’s `alloc: null`, and `counters: []`.
These are existing explicit unavailable observations, not measured zeros.
Keep all these fields and all original command/version/budget/truncation,
wall count/total/self/min/max/p50/p95/p99 and full stage/pass/block vectors.
The complete ten-span expected JSON in `tests/spolvero_timing.rs` stays exact.
No schema decision or invented allocation/counter measurement is proposed.

Native callers retain actual profiler span/counter snapshots and the exact
caller-supplied `Option<AllocationSnapshot>`. Every available allocation field,
counter value, attribution and native wire claim remains present. Existing
Carton allocation/schema laws and the CLI/SFC exporter callers stay mandatory.

## Proposed provider then genuine consumer

1. Keep `export_report(profiler: &Profiler, options: &ProfileExportOptions)`
   source compatible. Preserve its current order: read spans, rank/map/cap them,
   then read counters and rank/map/cap them. Factor those existing pure assembly
   operations into a bounded module shared by both entry points.
2. Add `export_report_from_snapshots` accepting the existing owned vectors
   `Vec<(&'static str, SpanAttribution, Metrics)>` and
   `Vec<(&'static str, CounterMetrics)>`, plus the same options. It consumes
   those inputs once through the sole ranking/truncation/serialization path.
   No extra snapshot wrapper, hidden host read or serialization roundtrip.
3. Only after that provider is reviewed, a genuine Curator child replaces its
   local native Profiler with a local `FxHashMap<(key, attribution), Metrics>`.
   The existing one loop over actual steps/walks records the existing Duration.
   Consume the map into the existing owned tuple vector; counters remain the
   real empty input and allocation remains the real unavailable `None`.
   This is the sole aggregate store, not a retained duplicate walk or timeline.

Preserve all ranking tiebreaks, percentile bounds, duration saturation,
max_spans/max_counters, explicit omitted counts, field order, pretty JSON and
trailing newline. Keep modules at the original 350-line cap through meaningful
assembly separation. Any true file moves use move-only commits and a bounded
TypeScript replay. Update exact old/new replay byte witnesses and only the
reviewed Curator host import; retain negative dependency/storage/import laws.

## Meaningful proof and delivery

Provider laws use full controlled spans/counters/allocation metadata to prove
snapshot/native export equality, deterministic full JSON, ownership after
source disposal, ordering, saturation and truncation. Existing wire references
and actual nested allocation/schema laws remain exact.

The consumer retains the entire original ten-span JSON, real timed pages,
six-read law and all eight plus 14 accepted timing laws. A real active-session
noninterference control must observe allocation tracking and counter retention,
not merely the profiler enable bit: local disable affects the global tracking
flag even if the separately global Profiler’s enabled bit remains true.
Use a single isolated instrumented test executable if needed to exclude libtest
allocation/reporting contamination; do not invent counter fields or use a
source-text test as a substitute for this actual behavioral proof.

Normal exact-source Actions and genuine native Stack/source-parent membership
precede admission; unchanged full suites, 104 probes × three and both actual
parent ratchets run in the protected queue before actual signed merges.
No local Rust/build/install or additional manual campaign is proposed.

Root must review this frozen plan before material implementation. Pair the
same bounded decision with #6834 and the central L0 record. Native Timer,
profiler TLS/locks, arena pool, recursion, IO and full std/platform isolation
remain unfinished; #6834 stays OPEN. Default products and 10x remain separate.

## Private provider source (implementation pending hosted proof)

The provider keeps the exact native exporter signature and its original
span-read/assemble then counter-read/assemble order. A bounded `assemble.rs`
contains the original rank/map/truncate/field construction and duration helper;
both public inputs call it. The new owned-input API consumes the same tuple
vectors and `ProfileExportOptions`, with no global-profiler side effect.
Four authored laws cover full nonzero supplied allocation/counter telemetry,
whole native/owned JSON equality after source disposal, complete attribution
tiebreaks/truncation, and unavailable/empty/duration-saturation contracts.
Controlled metadata is a caller fixture, not actual allocator measurement.
All original Carton wire/nested-allocation/schema, native timing and Curator
complete JSON/page/profile laws are unchanged and still required on Actions.
The exact source replay witness covers the new assembly body and keeps every
old malformed caller, dependency, golden-literal and gate rejection. The new
assembly mutation control refuses changed bytes before any replay writes.
The Curator child is not implemented in this provider; its real allocator
noninterference and whole output proof remain separate pending work.
