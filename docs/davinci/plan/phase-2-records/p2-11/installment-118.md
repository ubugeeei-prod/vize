# P2-11 Installment 118 - Disabled Static Hoist Routing

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5818](https://github.com/ubugeeei-prod/vize/pull/5818), merged
> 2026-09-06 as `90eb689e2`.

This installment lets production selection route supported
`hoist_static: false` DOM compiles through S2. The S2 emitter now carries the
published option instead of assuming the transform default, so static trees,
component slots, slot fallbacks, template branches and nested `v-once` shapes
avoid synthetic hoist declarations while still matching the compatibility
render bytes.

## Evidence

`crates/vize_atelier_dom/tests/davinci_s2_hoist_static_selector.rs` compares
disabled-hoist output against the compatibility lane and asserts that the
selected compile records the S2 DOM profiling counter. The option matrix and
production-boundary tooling keep `hoist_static` classified as projected into
`DomEmitOptions` rather than an accidental default.

This installment does not tick P2-11. The full production-lane switch remains
open because opaque custom-element predicates, unsupported option shapes and
the explicit legacy flag still require the compatibility path.
