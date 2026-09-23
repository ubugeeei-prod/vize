# P2-11 Installment 96 - Constant Text Runs

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5648](https://github.com/ubugeeei-prod/vize/pull/5648), merged
> 2026-09-04 at `7b002f744`.

This installment evaluates the constant-text rule per text-run part instead
of deciding from the whole run. Mixed text can contain static text beside one
or more interpolations, and the shipped lane applies the constant shortcut at
the part that owns the expression.

## Evidence

`crates/vize_atelier_dom/tests/davinci_s2_constant_text_run.rs` covers mixed
text and interpolation runs against the shipped DOM output. The child emitter
now keeps each part's constant decision local to that part.

This installment does not tick P2-11. The production-lane switch remains
open, and the old DOM lane is still the shipped compile path.
