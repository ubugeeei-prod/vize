# P2-11 Installment 6 — static-name `v-bind` (2026-08-23)

> [!NOTE]
> Part of the [P2-11 series record](../p2-11.md), split per installment
> under the 350-line source budget.

[#4657](https://github.com/ubugeeei-prod/vize/pull/4657) —
`71e150dd6917ba2f59eb35ba8f832a6c4c4ddf88` on `origin/main`.

## What landed

Static-name `ui.bind` (`:class` / `:style` / `:id` and other named
props) with the shipped patch flags (`CLASS` / `STYLE` / `PROPS` +
`dynamicProps`). Static `class` + `:class` merges through
`_normalizeClass([...])` in source order.

Dual-run battery covers dynamic class/id/style, class+interpolation,
static+dynamic class, and hyphenated binds.

Reopens the bind increment after #4653 was accidentally merged into a
parent feature branch. #4653 is **not** an `origin/main` installment
SHA.

## Named remainder after this increment

Object-spread `v-bind`, events, modifiers, `ref`, and static+dynamic
**style** stay `Unsupported` (`props.rs` still names the style merge
"next installment").
