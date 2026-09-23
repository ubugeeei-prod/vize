# P2-11 Installment 109 - S2 DOM Section Boundaries

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5771](https://github.com/ubugeeei-prod/vize/pull/5771), merged
> 2026-09-05 as `76a5323ab`.

This installment lets the S2 DOM path return the source-section boundaries that
inline SFC and custom-elements callers need. The production selector no longer
treats section-emitting requests as an automatic compatibility-path boundary
when the other S2 requirements are satisfied.

The section data is attached without adding publish-time dependencies from
`vize_atelier_dom` to unpublished Davinci stage crates. The S2 emitter returns
the body and prefix spans through the existing stage-output path, and the SFC
compile wrapper maps those spans back into the generated code sections expected
by the public DOM compile surface.

## Evidence

`crates/vize_atelier_dom/tests/davinci_s2_production_selector.rs` keeps the
selector boundary pinned: inline SFC and custom-elements compiles that need
sections can use S2, while source-map, comment-preserving and unsupported
option-shape requests continue to use the compatibility path.

Focused gates:

```sh
cargo test -p vize_atelier_dom --test davinci_s2_production_selector
cargo test -p vize_atelier_dom -p vize_atelier_sfc -p vize_s1_to_s2 --tests
cargo fmt --all -- --check
git diff --check
```

This installment does not tick P2-11. The full production-lane switch remains
open because source maps, comments, unsupported option shapes and the explicit
legacy flag still require the compatibility path.
