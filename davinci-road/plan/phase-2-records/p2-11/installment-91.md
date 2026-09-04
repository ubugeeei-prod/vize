# P2-11 Installment 91 - Unref Helper Order

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5635](https://github.com/ubugeeei-prod/vize/pull/5635), merged
> 2026-09-04 at `c1e5c7245`.

This installment places `_unref` with the transform-time helpers instead of
the later codegen-use list. Inline setup bindings can discover `_unref` only
while the emit walks the body, but the shipped lane orders that helper as if
the transform registered it before codegen. The S2 buffer now keeps that
preference without changing the default lane.

## Evidence

`crates/vize_atelier_dom/tests/davinci_s2_unref_helper_order.rs` pins the
helper preamble order byte-for-byte against the shipped DOM lane, including
the inline setup cases that introduced the late `_unref` mark.

This installment does not tick P2-11. The production-lane switch remains
open, and the old DOM lane is still the shipped compile path.
