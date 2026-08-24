# P2-11 Installment 19 — static-name `v-bind` modifiers (2026-08-24)

> Part of the [P2-11 series record](../p2-11.md), split per installment.

[#4803](https://github.com/ubugeeei-prod/vize/pull/4803) moved
static-name `v-bind` modifiers out of the S2 DOM emit lane's local
`Unsupported` bucket. Davinci now realizes `.camel`, `.prop`, `.attr`,
and the dot shorthand into the same DOM prop-key spellings as the
shipped lane:

- `.camel` camelizes the static key before emission.
- `.prop` prefixes the emitted key with `.` and preserves the shipped
  `NEED_HYDRATION` patch-flag bit.
- `.attr` prefixes the emitted key with `^`.
- `.foo="bar"` follows the lowering's parser-mirrored dot shorthand and
  emits exactly like `:foo.prop="bar"`.

The implementation keeps raw S2 names separate from realized DOM keys:
`emit/props_bind.rs` now owns the static bind-key transform and callers
choose ordinary props casing or the `<slot>` outlet camelizing rule. The
same helper feeds ordinary props, `mergeProps` dynamic-prop tracking, and
slot outlet props, so the modifier rules do not drift across the three
emit sites.

The durable witnesses are:

- `crates/vize_ricalco/tests/emit_bind_modifiers.rs` — direct S2 emit
  snapshots for `.camel`, `.prop`, `.attr`, and the dot shorthand.
- `crates/vize_atelier_dom/tests/davinci_s2_bind_modifiers.rs` — eleven
  S2-vs-shipped DOM lane byte-for-byte comparisons covering native
  elements, components, `mergeProps`, `v-if`, `v-for`, and slot outlets.

This installment does not tick P2-11. Filters, dynamic-argument
`v-bind` names/modifiers, and local slot/outlet guard-only shapes remain
in the named unsupported list.
