# P2-11 Installment 21 — Vue 2 pipe filters (2026-08-25)

> Part of the [P2-11 series record](../p2-11.md), split per installment.

[#4860](https://github.com/ubugeeei-prod/vize/pull/4860) moved Vue 2 pipe
filters out of the S2 DOM emitter's local refusal surface. The merge commit is
`d681caa7ae1d16e78ad30113af634fc231227375`.

The legacy-sugar pass rewrites `vue.filter` expression payloads into
`_filter_*` calls and records first-seen filter assets for `_resolveFilter`.
The emitter now matches the shipped DOM lane for filter interpolation chains,
filter arguments, dashed and dollar-prefixed filter names, static `v-bind`
values, component default slots, and slot outlet props. Vue 3 continues to
treat `|` as a normal expression operator rather than a filter.

The durable current witnesses are:

- [`emit_filters.rs`](../../../../crates/vize_ricalco/tests/emit_filters.rs)
  — direct S2 emission snapshots for filter assets, wrapped calls, and Vue 3
  non-filter behavior.
- [`davinci_s2_filters.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_filters.rs)
  — S2-vs-shipped byte-for-byte coverage under the legacy feature.

This installment does not tick P2-11. Local slot/outlet guard-only shapes
remain, and the old DOM lane is still the production path. The task-level
corpus, patch-flag, allocation-budget and publish-graph gates remain open.
