# P2-11 Installment 87 - TypeScript Templates

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5621](https://github.com/ubugeeei-prod/vize/pull/5621), merged into
> the installment-84 branch and carried to `origin/main` by
> [#5615](https://github.com/ubugeeei-prod/vize/pull/5615) at `e9923c0f8`.
> [#5623](https://github.com/ubugeeei-prod/vize/pull/5623) keeps the lane
> inside the stage library's own gates; the no-std ledger records the edge
> in [#5624](https://github.com/ubugeeei-prod/vize/pull/5624).

`is_ts`: template expressions are TypeScript, so each is type-erased before
the identifier pass reads it. `emit::prefix::typescript` ports the oxc
round-trip - wrap as `const _expr_ = (…);`, parse as TS, transform, print,
slice - and passes the original text through on every refusal, detector false
positives included.

Three further divergences it exposed:

- the transform's scope chain is seeded with croquis' `JS_UNIVERSAL_GLOBALS`,
  a **wider** set than `is_global_allowed`, so `Intl.x()` stays bare while
  `Zork.x()` is prefixed - not a TypeScript property at all;
- `process_inline_handler` checks _function_ shape on the stripped text but
  _reference_ shape on the node's own bytes;
- every downstream reader - patch-flag staticness, the hoist decision, the
  hoisted text - sees the erased text. `props::ts_view` is that view, and the
  static/hoist path threads an `is_ts` flag to it.

The lane needs `std` (oxc's `Transformer::new` takes a `&Path`), so it is an
**opt-in `typescript` feature**: the crate keeps its literal `#![no_std]` and
puts `std` behind `#[cfg(feature = …)] extern crate std;`, and an `is_ts` emit
without the feature refuses with `TypeScriptLaneUnavailable` rather than
silently emitting un-erased TypeScript.
