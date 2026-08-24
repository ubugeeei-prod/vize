# P2-11 Installment 17 — `.native` event sugar (2026-08-24)

> Part of the [P2-11 series record](../p2-11.md), split per installment.

[#4792](https://github.com/ubugeeei-prod/vize/pull/4792) moved
static-name `v-on` `.native` out of the S2 DOM emit lane's local
`Unsupported` bucket. The modifier is Vue 2 event sugar: the emitter
accepts it but strips it from event-key calculation and handler wrappers,
matching the shipped lane.

One compatibility wrinkle is pinned because it is easy to accidentally
"simplify" away: `.native` still makes the shipped lane choose the
multiline inline-handler prop shape and add `NEED_HYDRATION`. Davinci now
keeps that patch-flag behavior while leaving the authored handler value
unwrapped.

The durable witness is
`crates/vize_atelier_dom/tests/davinci_s2_native_on.rs`: twelve
S2-vs-shipped DOM lane comparisons covering native elements, components,
`v-if`, `v-for`, duplicate handlers, option modifiers, system/key
modifiers, and inline handler expressions. The comparison is
byte-for-byte including helper usage, patch flags, and dynamic-props
arrays.

This installment does not tick P2-11. Filters, bind modifiers,
static+dynamic `style` merge, and local slot/outlet guard-only shapes
remain in the named unsupported list.
