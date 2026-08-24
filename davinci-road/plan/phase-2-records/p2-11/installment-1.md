# P2-11 Installment 1 — name the DOM lane flag (2026-08-23)

> [!NOTE]
> Part of the [P2-11 series record](../p2-11.md), split per installment
> under the 350-line source budget.

[#4644](https://github.com/ubugeeei-prod/vize/pull/4644) —
`76c8c838e524635d7d8a58a9eb059a06dec8b933` on `origin/main`.

## What landed

The first P2-11 PR. Independent of the P2-10 stack. Names
`VIZE_DAVINCI_DOM` in `vize_ricalco::dom` as `DOM_LANE_FLAG` so the
phase-2 exit-gate deletion grep has one home (charter #26), the same
shape as `VIZE_DAVINCI_TRANSFORM`.

Ricalco is `no_std` and reads no environment. Value `legacy` is what
will disarm the S2 dual-run; the comparator that honors it lands with
the emit, not here. The published `vize_atelier_dom` graph is
untouched. The program decision in `phase-2.md` (publish / fold /
feature-gate the Davinci crates) is still open.

Witness: `cargo test -p vize_ricalco --lib dom_lane` pins the recorded
name.

## What this increment does not do

No emit. No dual-run. No shipped compile-path change. The box in
`phase-2.md` stays open.
