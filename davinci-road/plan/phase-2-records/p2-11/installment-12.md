# P2-11 Installment 12 — static-name components (2026-08-24)

> [!NOTE]
> Part of the [P2-11 series record](../p2-11.md), split per installment
> under the 350-line source budget.

[#4685](https://github.com/ubugeeei-prod/vize/pull/4685) —
`d7f42cca1e7d8d43e63835ccb896e7341c8d83ab` on `origin/main`.

## What landed

Static-name components from S2 (`resolveComponent` / `createVNode` /
`createBlock`) match the shipped DOM lane byte-for-byte, including
`v-if`, `v-for`, named binds, events, and object-spread `v-bind`.
Dual-run covers empty / nested / kebab components plus the
merge-props cases already on main.

## Named remainder after this increment (current)

Slots, builtins (`Transition`, `Teleport`, …), and `<component :is>`
stay `Unsupported` this installment. The `emit.rs` running list is
now: object `v-on`, `.native`, template fragments, filters, slots,
and builtins. See the [series remainder](../p2-11.md#named-unsupported-remainder-after-4685).

The series box stays open. No compiler behavior changed in the
record PR that adds this file.
