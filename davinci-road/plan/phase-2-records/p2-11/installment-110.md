# P2-11 Installment 110 - In-Tag DOM Option Routing

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5777](https://github.com/ubugeeei-prod/vize/pull/5777), merged
> 2026-09-05 as `0eae46139`.

This installment pins that in-tag comment preservation remains outside the S2
DOM production selector. `experimental_in_tag_comments` is an explicit
comment-preserving output contract, so the compatibility DOM lane still owns it
until the S2 emitter carries that option with byte-identical behavior.

The witnesses cover both production selection and the unsupported-profile
surface, preventing an option-router regression from quietly presenting S2 as
production-ready for a shape it still does not emit.

## Evidence

`crates/vize_atelier_dom/tests/davinci_s2_production_selector.rs` keeps the
production selector on the compatibility path for the in-tag option, while
`crates/vize_atelier_dom/tests/davinci_s2_profile_unsupported.rs` records the
same boundary in the unsupported-profile catalogue.

Focused gates:

```sh
cargo test -p vize_atelier_dom --test davinci_s2_production_selector --test davinci_s2_profile_unsupported
vp exec node --test tests/tooling/davinci-dom-production-boundary.test.ts
git diff --check
```

This installment does not tick P2-11. It narrows the selector truth table, but
the full production-lane switch remains open.
