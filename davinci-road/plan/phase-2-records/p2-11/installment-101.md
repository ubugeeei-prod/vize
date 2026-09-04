# P2-11 Installment 101 - Runtime Global Profiling

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: _pending - the number, merge date and squash SHA are filled in at
> merge, as every prior installment's line was._

This installment connects installment 84's custom runtime global-name emitter
parity witness to the production-side S2 selector. Adapter-provided
`CodegenOptions::runtime_global_name` is not a DOM compiler option, so it can
quietly lose S2 coverage unless the profiled compile path exercises the same
projection that `DomEmitOptions` already exposes.

With source maps disabled, a custom runtime global compile now has a direct
machine gate: it must enter S2 profiling, record the `davinci.s2_dom.*`
counters and stay byte-identical with the compatibility source-map compile.
The source-map request remains the compatibility boundary because S2 still
emits no map.

## Evidence

`crates/vize_atelier_dom/tests/davinci_s2_profile.rs` now asserts that a
source-map-free DOM compile using `runtime_global_name = "RuntimeVue"` stays on
S2 under profiling and produces the same preamble and render body as the
compatibility source-map compile.

This installment does not tick P2-11. The production-lane switch remains
open, and the old DOM lane is still the shipped non-profiled compile path.
