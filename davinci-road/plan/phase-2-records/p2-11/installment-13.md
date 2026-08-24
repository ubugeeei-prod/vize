# P2-11 Installment 13 — object-spread `v-on` (2026-08-24)

> Part of the [P2-11 series record](../p2-11.md), split per installment.

[#4735](https://github.com/ubugeeei-prod/vize/pull/4735) emits the
nameless `ui.on` form (`v-on="handlers"`) as `_toHandlers(expr, true)`.
The `true` argument is the shipped lane's `handlerOnly` flag and is
preserved both for a lone object listener spread and when the listener
object is one argument in `_mergeProps(...)`.

The helper catalogue moved into `emit/helper.rs` so `_toHandlers` keeps
the same import rank as `normalizeProps`, `guardReactiveProps`, and
`mergeProps`. Witnesses cover the lone form, merge order beside static
attrs / named events / object `v-bind`, and component cases through
`vize_ricalco` emit tests plus the atelier_dom S2 comparison battery.

Still not handled here: `.native`, slots, template fragments, filters,
builtins, `<component :is>`, dynamic keys, destructured `v-for` aliases,
`v-model`, and custom directives. Those remain later P2-11 increments.
