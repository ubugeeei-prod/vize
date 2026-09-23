# P2-11 Installment 100 - Scoped Compile Profiling

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5681](https://github.com/ubugeeei-prod/vize/pull/5681), merged
> 2026-09-04 as `8b91e071b`.

This installment connects installment 99's `scope_id` emitter parity witness to
the production-side S2 selector. Direct scoped DOM compiles no longer stay on
compatibility codegen when profiling is enabled: with `source_map` disabled,
the compiler routes them through the S2 DOM emitter and records the same
`davinci.s2_dom.*` counters as unscoped output.

The source-map request remains the compatibility boundary. S2 still emits no
map, so the production switch is narrowed without weakening the documented
source-map contract or changing the shipped non-profiled lane.

## Evidence

`crates/vize_atelier_dom/tests/davinci_s2_profile.rs` now asserts that a
direct `scope_id` compile enters S2 profiling and stays byte-identical with the
compatibility source-map compile. This keeps `scope_id` in the measured
production option surface instead of as an emitter-only witness.

This installment does not tick P2-11. The production-lane switch remains
open, and the old DOM lane is still the shipped compile path.
