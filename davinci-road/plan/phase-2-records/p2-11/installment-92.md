# P2-11 Installment 92 - Inline Template Refs

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5636](https://github.com/ubugeeei-prod/vize/pull/5636), merged
> 2026-09-04 at `741f05750`.

This installment resolves `ref` values in the inline S2 DOM path through the
same setup-binding rules as the shipped lane. Template refs now pass through
the prefix scope and prop emission paths that know whether a binding is a
setup ref, a setup maybe-ref, or an ordinary context member.

## Evidence

`crates/vize_atelier_dom/tests/davinci_s2_inline_template_ref.rs` compares
static and bound template-ref output against the shipped lane for the inline
configuration. The new scope helpers cover the prop and props-object arms
that can carry refs.

This installment does not tick P2-11. The production-lane switch remains
open, and the old DOM lane is still the shipped compile path.
