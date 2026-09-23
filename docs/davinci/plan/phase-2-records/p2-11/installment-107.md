# P2-11 Installment 107 - Shared Model And Outlet Batteries

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5764](https://github.com/ubugeeei-prod/vize/pull/5764), merged
> 2026-09-05 as `4857d3e75`.

This installment moves the model and slot-outlet byte-parity witnesses into the
shared DOM battery support tree, then reuses those same cases from the standalone
S2 tests and the production-option family witness. The option-family matrix now
checks that the broad model and outlet surfaces still match the shipped output
when `cache_handlers` and `scope_id` are active together.

The change is intentionally test-structural: it makes the production selector's
next switch harder to under-test without changing emitted bytes on its own.

## Evidence

`crates/vize_atelier_dom/tests/support/battery/model.rs` and
`crates/vize_atelier_dom/tests/support/battery/outlets.rs` own the shared
cases. `crates/vize_atelier_dom/tests/davinci_s2_model.rs`,
`crates/vize_atelier_dom/tests/davinci_s2_outlets.rs` and
`crates/vize_atelier_dom/tests/davinci_s2_option_families.rs` consume them.

Focused gates:

```sh
cargo fmt --all -- --check
cargo test -p vize_atelier_dom --test davinci_s2_model --test davinci_s2_outlets --test davinci_s2_option_families
rust-script tools/commands/davinci/assertion-lint.rs
rust-script tools/commands/ci/source-file-lengths.rs --check --base-ref origin/main
git diff --check
```

This installment does not tick P2-11. The full production-lane switch remains
open, and the old DOM lane remains the compatibility path.
