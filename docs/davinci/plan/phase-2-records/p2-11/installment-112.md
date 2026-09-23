# P2-11 Installment 112 - SFC Namespace Selection Hardening

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5779](https://github.com/ubugeeei-prod/vize/pull/5779), merged
> 2026-09-05 as `297e3f1e`.

This installment hardens SFC namespace selection after the initial S2 admission.
The scanner stores the resolved namespace on the open-tag stack, skips complete
comments and declarations, scans a full tag while respecting quoted text, and
keeps root HTML names such as `<title />` on the HTML compatibility surface.

It also keeps the generated asset section boundary aligned with component-asset
compatibility output, so namespace admission does not produce stale section
metadata.

## Evidence

`crates/vize_atelier_dom/tests/davinci_s2_sfc_namespace_selector.rs` pins custom
foreign descendants, non-ASCII quoted attribute text, and root HTML title
compatibility.

Focused gates:

```sh
cargo test -p vize_atelier_dom --test davinci_s2_sfc_namespace_selector --test davinci_s2_production_selector --test davinci_s2_profile --test davinci_s2_dom_namespace
cargo test -p vize_s1_to_s2
cargo clippy -p vize_s1_to_s2 --lib -- -D warnings
cargo clippy -p vize_atelier_dom --tests -- -D warnings
cargo fmt --all -- --check
git diff --check
```

This installment does not tick P2-11. It removes selector fragility around the
new SFC namespace route, but the full production-lane switch remains open.
