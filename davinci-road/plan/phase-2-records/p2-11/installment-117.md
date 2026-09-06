# P2-11 Installment 117 - Bare Static Style Merges

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5821](https://github.com/ubugeeei-prod/vize/pull/5821), merged
> 2026-09-06 as `219f5994a`.

This installment admits valueless static `style` attributes beside dynamic
`:style` in S2 DOM emit. The earlier style-merge witness covered valued static
styles and left bare static styles in the unsupported bucket; the production
path now matches the shipped lane by treating the bare static attribute as an
empty static style object before merging the dynamic binding.

## Evidence

`crates/vize_atelier_dom/tests/davinci_s2_style_merge.rs` compares the
bare-static-plus-dynamic cases against the shipped DOM output, and the
`vize_s1_to_s2` static-prop tests pin the direct emitter behavior. The
unsupported census and catalogue retire the former refusal so the gap cannot
silently reappear.

This installment does not tick P2-11. The full production-lane switch remains
open because opaque custom-element predicates, unsupported option shapes and
the explicit legacy flag still require the compatibility path.
