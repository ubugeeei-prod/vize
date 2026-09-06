# P2-11 Installment 120 - Legacy Selection Guard

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5829](https://github.com/ubugeeei-prod/vize/pull/5829), merged
> 2026-09-06 as `5fea9fdc0`.

This installment makes the remaining compatibility selection explicit. Vue 2
dialects and `VIZE_DAVINCI_DOM=legacy` compiles stay on the old DOM lane until
the production-switch exit gate deletes the flag and reviews the release-graph
boundary. The S2 selector now receives a lane value instead of reading an
implicit environment default at the support boundary.

## Evidence

`crates/vize_atelier_dom/tests/davinci_s2_legacy_selector.rs` compiles a Vue 2
legacy template and a Vue 3 template under `VIZE_DAVINCI_DOM=legacy`, then
asserts both return compatibility sections without recording the
`davinci.s2_dom.files` profiling counter. The supporting
`stage_options` tests pin `legacy`, `s2` and unset environment values to their
selection lanes and reject S2 support when the legacy lane is selected.

This installment does not tick P2-11. The full production-lane switch remains
open because opaque custom-element predicates, unsupported option shapes and
the explicit legacy flag deletion still require a reviewed compatibility-lane
retirement.
