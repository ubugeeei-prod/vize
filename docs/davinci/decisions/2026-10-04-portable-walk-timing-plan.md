# Caller-owned fused-walk timing — private source plan

Issue: [#6834](https://github.com/ubugeeei-prod/vize/issues/6834).
Planning baseline: signed actual main
[`76f93fe1e3c46ac8c86157a520302e26c50ac871`](https://github.com/ubugeeei-prod/vize/tree/76f93fe1e3c46ac8c86157a520302e26c50ac871),
read on 2026-10-04 08:18 UTC. This is a review proposal, with no implemented,
executed, published, queued or merged source credit.

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

## Smallest implementation proposal

Extract the existing open-walk state and lifecycle rules into one L0 helper,
provisionally `WalkTiming<Mark>`, under `pass/observer/timing/`. It owns only the
existing optional open mark and its `SpanAttribution`. `Mark` is supplied by the
caller; the helper never reads a clock, reaches the global profiler, allocates,
serializes or traverses nodes.

Proposed operations are restart/discard, group-entry with a lazy
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

## Paired record and scope

After source-plan review, pair the implementation decision with #6834 and the
central L0 paragraph in the same genuine source change. Do not publish this
planning record as a receipt-only PR. Preserve allocator, export/schema,
fixtures, all instruction ceilings and other owners' source reservations.
Actual product defaults, all remaining history gates, broader 10x/performance,
full #6834 portability and whole Davinci completion remain unfinished.
