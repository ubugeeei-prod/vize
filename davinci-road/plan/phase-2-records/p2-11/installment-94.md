# P2-11 Installment 94 - Cached Props Multiline

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5653](https://github.com/ubugeeei-prod/vize/pull/5653), merged
> 2026-09-04 at `d4e1ce25b`.

This installment keeps cached props multiline when a cached static prop value
is not a simple expression. The shipped lane breaks the object layout in that
case so nested caches and non-simple values stay readable and byte-identical;
S2 now mirrors that layout decision in both cached and static prop emit.

## Evidence

`crates/vize_atelier_dom/tests/davinci_s2_cached_props_multiline.rs` compares
the affected cached-props shapes against the shipped lane. The regression is
covered at the object-emitter level and through static prop emission.

This installment does not tick P2-11. The production-lane switch remains
open, and the old DOM lane is still the shipped compile path.
