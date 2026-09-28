# vize_extension_sdk

The SDK for Vize extension-contract guests: the canonical `vize:contracts`
WIT package (`wit/`) and its released surface history (`versions/`),
its bindings, the capability-handshake constants, writers for the L1 and L2
pages the host accepts, and the runtime an import-free `no_std`
`wasm32-wasip2` guest provides itself. It depends on no Vize implementation
crate.

The canonical layer APIs use `L1_PAGE_SCHEMA`, `L2_PAGE_SCHEMA`,
`L3_PAGE_SCHEMA` and `pages::{l1, l2, l1_page, l2_page}`. The original `S`
constants and `pages::{s1, s2, s1_page, s2_page}` remain aliases. Wire page
names (`s1-page@1`, `s2-page@1`, `s3-page@1`) and WIT fields stay unchanged.

Contract versioning follows
[the compatibility policy](https://github.com/ubugeeei-prod/vize/blob/main/docs/davinci/contracts-compat-policy.md).
Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

## Typed expression guests

Enable the `typed-expression` feature to select the
`typed-expression-dialect` export world and its
`export_typed_expression_dialect!` macro. Implement `typed_handshake::Guest`
and `typed_bindings::exports::vize::contracts::typed_expression_analysis::Guest`.
`typed_capability()` offers the typed world's exact required features. The default selects the existing
input world. World selection avoids duplicate public handshake macros in
`wit-bindgen`; each component exports one selected world.

Typed bindings require a producer-supplied dialect signature. Local bindings
shadow the block environment only inside their own expression. A missing or
unknown type must be refused rather than guessed. The world carries the same
versioned facts and projection pages as the original expression world and
adds the `typed-environment@1` handshake requirement.

## License

MIT
