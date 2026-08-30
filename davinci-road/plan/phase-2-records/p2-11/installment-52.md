# P2-11 Installment 52 - Opaque Bind And Empty Text Edges

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5390](https://github.com/ubugeeei-prod/vize/pull/5390), merged
> 2026-08-30 at `16a3fc970`.

This installment emits opaque bind values and empty text edges without losing
the shipped DOM shape. Opaque expressions keep their pessimal source contract
from P2-5b while still rendering through the same prop, child, fragment and
outlet paths the old emitter used.

The durable witnesses are:

- [`davinci_s2_expression_residuals.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_expression_residuals.rs)
  - compares the expression residuals against shipped DOM output.
- [`props_value.rs`](../../../../crates/vize_s1_to_s2/src/emit/props_value.rs)
  and [`children.rs`](../../../../crates/vize_s1_to_s2/src/emit/children.rs)
  - own the prop and text realization paths.
- [`emit_unsupported_census.rs`](../../../../crates/vize_s1_to_s2/tests/emit_unsupported_census.rs)
  - keeps unsupported expression classes explicitly counted.

This installment does not tick P2-11. It narrows expression residuals; the
hydrated corpus evidence and production-lane switch remain open.
