# P2-11 Installment 111 - SFC Namespace Templates

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5778](https://github.com/ubugeeei-prod/vize/pull/5778), merged
> 2026-09-05 as `8f3d843c`.

This installment admits SFC templates whose namespace can be selected before
DOM emission. The S2 path no longer rejects supported HTML/SVG/MathML template
roots merely because the caller came through the SFC compile wrapper.

The change keeps the compatibility lane for source maps, comment-preserving
requests and unsupported option shapes; it widens only the S2-safe namespace
surface.

## Evidence

`crates/vize_atelier_dom/tests/davinci_s2_sfc_namespace_selector.rs` records the
SFC namespace selector behavior, and the production selector tests keep the
fallback boundary visible.

Focused gates:

```sh
cargo test -p vize_atelier_dom --test davinci_s2_sfc_namespace_selector --test davinci_s2_production_selector
cargo fmt --all -- --check
git diff --check
```

This installment does not tick P2-11. It moves another production SFC entry
shape onto S2, while the explicit compatibility blockers remain.
