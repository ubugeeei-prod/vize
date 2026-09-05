# P2-11 Installment 108 - Source-Map-Free Production Selector

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5765](https://github.com/ubugeeei-prod/vize/pull/5765), merged
> 2026-09-05 as `8be4517e`.

This installment moves supported source-map-free DOM compiles onto the S2 path
without depending on profiling being enabled. The production selector now routes
direct DOM compiles through S2 when the request has no source map, no comment
preservation and no unsupported option shape; source-map, comment-preserving and
hidden SFC/custom-elements section requests stay on the compatibility path until
S2 can emit those contracts.

The same PR hardens the switched path by removing observer overhead from the
unobserved S2 emitter, preserving UTF-8 while stripping slot-scope prefixes, and
guarding deep S2 emit/folio recursion on small stacks. The DOM traversal and
expression-reparse floors were updated to the measured S2 selector values, and
the owned-storage inventory was refreshed after reviewing the new stack-guard
sites.

## Evidence

`crates/vize_atelier_dom/tests/davinci_s2_production_selector.rs` asserts the
selector boundaries: supported source-map-free compiles use S2, source maps and
comments use compatibility, and SFC section-emitting compiles remain on
compatibility until S2 section offsets land. `crates/vize_s2/tests/folio_deep_stack.rs`
pins the deep-folio stack witness.

Focused gates:

```sh
RUST_BACKTRACE=1 cargo test -p vize_atelier_dom -p vize_s1_to_s2 -p vize_s2 --tests
cargo test -p vize_s2 --test folio_deep_stack -- --nocapture
rust-script tools/commands/davinci/assertion-lint.rs
rust-script tools/commands/ci/source-file-lengths.rs --check --base-ref origin/main
cargo fmt --all -- --check
git diff --check
node --test tests/tooling/davinci-traversal-budgets.test.ts tests/tooling/davinci-storage-policy.test.ts
```

This installment does not tick P2-11. The full production-lane switch remains
open because source-map/comment/unsupported-option compiles and the explicit
legacy flag still require the compatibility path.
