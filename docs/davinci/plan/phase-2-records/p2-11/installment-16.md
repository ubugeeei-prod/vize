# P2-11 Installment 16 — template refs (2026-08-24)

> Part of the [P2-11 series record](../p2-11.md), split per installment.

[#4788](https://github.com/ubugeeei-prod/vize/pull/4788) moved template
refs into the S2 DOM emit lane. Static `ref`, dynamic `:ref`, and the
`ref_for: true` marker inside `v-for` now mirror the shipped lane for
native elements and components.

The patch-flag rule is intentionally weird because it is Vue-compatible:
dynamic refs use `NEED_PATCH` instead of the dynamic-props list, but
static refs add `NEED_PATCH` only when no stronger prop patch flag already
covers the node. Object-spread `v-bind` keeps the same rule:
`:ref` beside a spread emits `FULL_PROPS, NEED_PATCH` with no `["ref"]`
dynamic-prop list.

The durable witness is
`crates/vize_atelier_dom/tests/davinci_s2_refs.rs`: eighteen
S2-vs-shipped DOM lane comparisons covering native refs, component refs,
`v-for` refs, keyed `v-for` refs, and object-spread combinations. The
comparison is byte-for-byte including helper usage, patch flags, and
dynamic-props arrays.

This installment does not tick P2-11. `.native`, filters, bind modifiers,
static+dynamic `style` merge, and local slot/outlet guard-only shapes
remain in the named unsupported list.
