# P2-11 Installment 53 - Trailing Block Comment Expressions

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5391](https://github.com/ubugeeei-prod/vize/pull/5391), merged
> 2026-08-30 at `e741ff65d`.

This installment admits expressions with trailing block comments at the shared
expression-retention boundary that feeds S2 DOM emission. The retained JS
surface now accepts the authored text the shipped emitter already handled,
removing a false unsupported edge from DOM namespace and expression witnesses.

The durable witnesses are:

- [`expression_retention.rs`](../../../../crates/vize_armature/tests/expression_retention.rs)
  - pins the parser-side retention behavior.
- [`expression_guard.rs`](../../../../crates/vize_carton/tests/expression_guard.rs)
  - keeps the guard decision aligned with retained expressions.
- [`davinci_s2_dom_namespace.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_dom_namespace.rs)
  - proves the DOM-facing witness no longer trips on this expression edge.

This installment does not tick P2-11. It closes a shared expression admission
edge; the hydrated corpus evidence and production-lane switch remain open.
