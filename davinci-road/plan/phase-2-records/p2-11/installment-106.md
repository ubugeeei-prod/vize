# P2-11 Installment 106 - Runtime Module Profiling

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: _pending - the number, merge date and squash SHA are filled in at
> merge, as every prior installment's line was._

This installment connects installment 84's custom runtime module-name emitter
parity witness to the production-side S2 selector. Module output can import
helpers from an adapter-provided `CodegenOptions::runtime_module_name`, and
that projection must stay covered by the profiled source-map-free DOM compile
path rather than only by direct emitter tests.

The source-map request remains the compatibility boundary because S2 still
emits no map. With source maps disabled, a custom runtime module compile now
enters S2 profiling, records the `davinci.s2_dom.*` counters and stays
byte-identical with the compatibility source-map compile.

## Evidence

`crates/vize_atelier_dom/tests/davinci_s2_runtime_module_profile.rs` asserts
that a module-mode DOM compile using
`runtime_module_name = "@scope/vue-runtime"` stays on S2 under profiling and
produces the same import preamble and render body as the compatibility
source-map compile.

Focused gate:

```sh
cargo test -p vize_atelier_dom --test davinci_s2_runtime_module_profile
```

This installment does not tick P2-11. The production-lane switch remains
open, and the old DOM lane is still the shipped non-profiled compile path.
