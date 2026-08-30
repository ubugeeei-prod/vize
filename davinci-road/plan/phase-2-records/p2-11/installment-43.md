# P2-11 Installment 43 - Dynamic Directive Argument Prefixing

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#4862](https://github.com/ubugeeei-prod/vize/pull/4862), merged
> 2026-08-30 at `fdaa8d165`.

This installment closes a dynamic directive argument gap shared by shipped DOM
codegen and the S2 DOM witness path. Compound `v-bind` and `v-on` argument
expressions now flow through the existing identifier visitor, so keys like
`:[prefix + suffix]`, `:[foo.bar]`, `:[keyOf(item)]` and
`@[prefix + suffix]` prefix every context identifier while preserving `v-for`
and slot locals.

The durable witnesses are:

- [`dynamic_bind_on_argument.rs`](../../../../crates/vize_atelier_core/tests/dynamic_bind_on_argument.rs)
  - pins shipped DOM codegen against Vue-compatible dynamic argument prefixing.
- [`davinci_s2_dynamic_bind_keys.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_dynamic_bind_keys.rs)
  - keeps S2 dynamic bind keys byte-identical to the shipped DOM lane.
- [`emit_dynamic_on_keys.rs`](../../../../crates/vize_s1_to_s2/tests/emit_dynamic_on_keys.rs)
  - keeps S2 dynamic `v-on` keys on the shipped helper shape, including
    compound names and local scope exceptions.

This installment does not tick P2-11. It closes the dynamic directive argument
prefixing witness gap, but the production-lane switch, hydrated
zero-divergence corpus run and remaining patch-flag program stay open.
