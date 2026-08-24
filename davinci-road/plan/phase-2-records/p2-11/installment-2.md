# P2-11 Installment 2 — static native HTML (2026-08-23)

> [!NOTE]
> Part of the [P2-11 series record](../p2-11.md), split per installment
> under the 350-line source budget.

[#4645](https://github.com/ubugeeei-prod/vize/pull/4645) —
`2e749a6cbeae362cf6aaffaa1957b33fbe3feb40` on `origin/main`.

## What landed

First S2 → DOM emit. Static native HTML elements (empty, text child,
nested) write the render function **directly from ops**. No relief
codegen-nodes (`NodeType` 13–20) are minted on this path.

Dual-run lives in `vize_atelier_dom` test space (dev-dep carve-out):
old `compile_template` vs S2 emit, **byte-for-byte**, including helper
order. `VIZE_DAVINCI_DOM=legacy` is named; a pinned comparison count
makes a silent disarm fail.

Home: `vize_ricalco::emit` (`emit.rs` + `emit/buf.rs` / `js.rs` /
`vnode.rs`). Public entry: `emit_dom` / `emit_dom_source`. Refusal is
`EmitError::{Diagnostics, Unsupported}` — never a panic.

## Named remainder after this increment

Bound attrs, interpolations, components, directives, and every other
non-static-native-HTML shape stay `Unsupported`. The old lane stays
the shipped compile path.
