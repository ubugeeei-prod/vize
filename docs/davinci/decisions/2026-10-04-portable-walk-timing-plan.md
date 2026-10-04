# Caller-owned fused-walk timing — private provider and consumer

Issue: [#6834](https://github.com/ubugeeei-prod/vize/issues/6834).
Planning baseline: signed actual main
[`76f93fe1e3c46ac8c86157a520302e26c50ac871`](https://github.com/ubugeeei-prod/vize/tree/76f93fe1e3c46ac8c86157a520302e26c50ac871),
read on 2026-10-04 08:18 UTC. The reviewed plan is now a private provider and its genuine Curator child.
No Rust execution, publication, queue or actual merge credit has been
established for either source.

## Readiness and concrete gap

The live issue is Stage 1, after closed #6833. Root reports no active clock or
timing owner. Existing allocator selection and profile export are already host
owned; this slice does not reopen those deliveries or replace product routes.

`vize_l0::profiler::Timer` still stores `std::time::Instant` at
`profiler/core.rs:112`. `TimingObserver` stores an open Timer and attribution,
opens it through the global profiler only at group entry, and closes it at group
exit. Pipeline restart and failure discard the open timer. Disabled profiling
must keep its relaxed atomic enable check and take no timestamp.

Curator already exposes `LadderClock = &dyn Fn() -> u64`. Its `PassWindows`
independently implements the same group entry, group exit and lead-pass identity
rules. It samples its actual host clock once before and once after each pass;
the existing three-pass/two-walk law uses six samples and expects the fused
walk 0..30 and barrier walk 40..50. Adding a second clock source or a separate
observer that reads more timestamps would change the observation.

## Bounded implementation decision

The provider extracts the existing open-walk state and lifecycle rules into
`WalkTiming<Mark>` at `pass/observer/timing/walk.rs`. It owns only the
existing optional open mark and its `SpanAttribution`. `Mark` is supplied by the
caller; the helper never reads a clock, reaches the global profiler, allocates,
serializes or traverses nodes.

Its operations are restart/discard, group-entry with a lazy
`FnOnce() -> Option<Mark>` factory, and group-exit returning the owned mark plus
its existing attribution. Non-entry passes must not invoke the factory;
non-exit passes must not consume the open mark. Attribution remains the stage
and lead pass of the existing `PassEvent`. Failure and restart discard the mark.
The helper is an observation, not source/artifact completion authority.

1. Provider plus existing native consumer: replace `TimingObserver`'s private
   `WalkSpan` state with this helper over the same `profiler::Timer`. Keep
   constructors, keys, counts, lazy `Profiler::timer`, stop/record calls,
   disabled behavior and existing export contracts. Do not generalize or move
   the entire profiler, Timer, TLS, locks or allocator in this change.
2. Genuine Curator child: reuse the helper over the already sampled `u64`.
   Replace its walk-start mirror, retaining the separate pass-start cell.
   Pass the exact existing before/after timestamps to the helper; preserve the
   actual clock call order/count, saturation, every step/walk, lead attribution,
   dumping-after-measurement, profile JSON and browser API. The existing
   borrowed host clock stays the only clock; add no clock trait or vtable.

The helper replaces both existing open-walk representations. It must not add a
second retained walk, a new pipeline stage, parse, walk, serialization, global
clock origin or native dependency on Carton. A Copy specialization may use the
current Cell discipline for Curator's scalar marks; no RefCell/lock is needed.

## Meaningful validation and delivery

Retain the actual native timing/export law and Curator's six-read law. Add
controlled caller marks with a counted lazy factory: fused/barrier attribution,
zero factory calls on non-entry or disabled starts, one returned mark per
completed walk, and exact discard on failure/restart. A non-Copy mark with a
drop counter proves custody and no retained/double-consumed mark. These are
lifecycle and real caller contracts, not tests of textual implementation shape.

Use configured automatic affected Actions and required aggregates, then the
protected full Rust/tooling/native corpus and all unchanged 104 instruction
probes three times with both ratchets. A two-PR provider/consumer split uses a
genuine GitHub native Stack and admits only its exact-source passing prefix.
No extra manual campaign, local Cargo/build/install or new workflow is proposed.

The existing browser build is `wasm32-unknown-unknown`; the Check portability
lane builds stage libraries on `wasm32-wasip2`, including no-default-features.
That lane is schedule/manual-only and does not establish no_std: L0 currently
has no no_std feature or `#![no_std]`. This proposal supplies clock-free walk
state and uses the real browser clock consumer. Bare-metal targets and complete
std/TLS/clock/global-profiler isolation remain unimplemented.

## Current private proof and paired record

Root read the full source plan and both existing consumer laws before authorizing
private implementation. The provider integrates the real native TimingObserver
and authors six controlled laws: actual fused/barrier callbacks, intermediate
laziness and single exit ownership, declined starts, actual-runner failure and
restart, interrupted pipeline restart, and exact drop of an abandoned non-Copy
mark. These laws have not been executed locally or on Actions yet. The original
native timing/export test is byte-exact; the concrete Instant Timer, global gate,
profiler/schema/budget and existing corpus files are unchanged.

The dependent Curator source consumes `WalkTiming<u64>` through its existing
Cell, replacing the separate walk-start mirror. Native helper bytes are exact
to the provider. It passes already sampled marks without a clock trait, another
timestamp, allocation or lock. Existing step/walk/page/export laws are retained;
the six-sample unit law additionally asserts its complete read count. Two new
real-callback laws retain complete original failure, discard on failure/restart
and the next actual fused window with exactly four new samples. Curator's
public full-ladder, page and profile/schema law files remain byte-exact.

Configured affected-Rust planning owns the provider files under vize_l0 and
selects reverse dependencies; the genuine child owns vize_curator. Existing
full protected Rust/tooling/native hooks and all104 instructions/both ratchets
stay mandatory. There is no workflow/selector/dependency change or new campaign.
Formatting, diff integrity, whole-file strict assertion scans, exact storage
identity and unchanged19-file consumer inventory are local source checks only. Publication and
native Stack membership await whole source review; root's finite release
sequencing currently holds queue admission.

Pair this same decision with #6834 and the existing central L0 paragraph in the
genuine source change. An issue-comment draft is prepared privately until
source review authorizes publication. No receipt-only PR is proposed. Preserve
allocator/export/schema/fixtures/ceilings and other owners' source reservations.
Actual product defaults, remaining history gates, broader10x/performance, full
#6834 portability and whole Davinci completion remain unfinished.
