# P2-11 Installment 95 - Constant Style Binding

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5641](https://github.com/ubugeeei-prod/vize/pull/5641), merged
> 2026-09-04 at `58afa0ca0`.

This installment teaches S2 DOM emit the shipped lane's constant-expression
shortcut for style bindings. A constant `:style` binding does not need
`normalizeStyle`; dynamic or opaque bindings still take the normal runtime
helper path.

## Evidence

`crates/vize_atelier_dom/tests/davinci_s2_constant_style_binding.rs` pins the
constant and non-constant style cases byte-for-byte. The new
`emit::constant_expr` helper centralizes the predicate so the style, prefix and
props-object paths agree.

This installment does not tick P2-11. The production-lane switch remains
open, and the old DOM lane is still the shipped compile path.
