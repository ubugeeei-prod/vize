# P2-11 Installment 18 — static+dynamic `style` merge (2026-08-24)

> Part of the [P2-11 series record](../p2-11.md), split per installment.

[#4796](https://github.com/ubugeeei-prod/vize/pull/4796) moved a
static `style` attribute beside static-name `:style` out of the S2 DOM
emit lane's local `Unsupported` bucket. Davinci now emits the shipped
lane's merged style array shape, preserving authored order between the
static declaration object and the dynamic expression.

The implementation keeps style serialization local to
`vize_s1_to_s2::emit`: valued static style attrs are admitted beside
`:style`, valueless static style attrs remain refused, CSS declarations
split only at semicolons outside function calls, and object-literal
dynamic style values keep the shipped lane's helper shape once they are
wrapped in a merged array.

The durable witness is
`crates/vize_atelier_dom/tests/davinci_s2_style_merge.rs`: eight
S2-vs-shipped DOM lane comparisons covering static-before-dynamic,
dynamic-before-static, object literals, CSS functions containing
semicolons, `v-if`, `v-for`, and `mergeProps` spread ordering. The
comparison is byte-for-byte including helper usage, patch flags, and
dynamic-props arrays.

This installment does not tick P2-11. Filters, bind modifiers, and local
slot/outlet guard-only shapes remain in the named unsupported list.
