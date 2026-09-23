# P2-11 Installment 93 - Merged Const Handler

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5640](https://github.com/ubugeeei-prod/vize/pull/5640), merged
> 2026-09-04 at `3ce792355`.

This installment applies the constant-handler rule after props have taken the
merged object path. The shipped lane does not decide handler const-ness only
on the direct `v-on` arm; a handler can move through the merged props builder
and still need the same cache and patch-flag decision.

## Evidence

`crates/vize_atelier_dom/tests/davinci_s2_merged_const_handler.rs` pins the
merged-props cases against the shipped output. The changes in `emit::merge`
and `emit::props` make the rule visible to both direct and merged event props.

This installment does not tick P2-11. The production-lane switch remains
open, and the old DOM lane is still the shipped compile path.
