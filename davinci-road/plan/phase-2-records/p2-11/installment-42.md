# P2-11 Installment 42 - S2 DOM Emit Allocations

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5360](https://github.com/ubugeeei-prod/vize/pull/5360), merged
> 2026-08-30 at `f659b7e4e`.

This installment pins the allocation budget for the late P2-11 S2 DOM emit
surface. The new `s1_to_s2_emit_p2_11_dom_surface` benchmark excludes parse,
lower and transform setup, then measures `emit_dom` over a fixture combining
dynamic bind modifiers, object bind/on modifiers, dynamic component
`v-model:[arg]`, slot outlet listeners, `v-show`, `v-html`, `v-text` and
`v-cloak`. The exact allocation gate is `allocs = 60`; wall time remains
report-only until a reference-runner baseline exists.

The durable witnesses are:

- [`davinci_storage.rs`](../../../../crates/vize_s1_to_s2/benches/davinci_storage.rs)
  - defines the emit-only benchmark window and the synthetic P2-11 DOM surface.
- [`budgets.toml`](../../budgets.toml)
  - registers `s1_to_s2_emit_p2_11_dom_surface` with `allocs = 60`.
- [`check.yml`](../../../../.github/workflows/check.yml)
  - runs the benchmark and `bench-compare` row in the Davinci allocation gate.
- [`davinci-budgets.test.ts`](../../../../tests/tooling/davinci-budgets.test.ts)
  - keeps the budget row reconciled with the bench source and asserts the
    exact allocation count.

This installment does not tick P2-11. It gates the S2 DOM emit allocation
surface, but the production-lane switch, hydrated zero-divergence corpus run
and remaining patch-flag program stay open.
