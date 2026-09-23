# P2-11 Installment 3 — static HTML attrs (2026-08-23)

> [!NOTE]
> Part of the [P2-11 series record](../p2-11.md), split per installment
> under the 350-line source budget.

[#4649](https://github.com/ubugeeei-prod/vize/pull/4649) —
`26bbb92bc953940bfe32ae154fe56d0b6cdb6a1e` on `origin/main`.

## What landed

Static native attributes, including hyphenated names and boolean empty
values. Root elements hoist the props object as `_hoisted_1` (shipped
`is_root` + `has_static_props`). Nested native children keep inline
props because `hoist_static_vnodes` is off without directives.

Dual-run battery vs `compile_template` stays byte-for-byte.

## Named remainder after this increment

Bound attrs remain `EmitError::Unsupported`. Interpolations, mixed
siblings, events, and components are still later installments.
