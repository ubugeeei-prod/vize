# vize_l2

`vize_l2` is Davinci's input-neutral semantic IR. Dialect-specific lowering
crates use its operations, provenance, scope, and invariant checks.

The optional `expression-queries` feature exposes the resolved uses in one
completed File through `query::Model`. Each result borrows the original
occurrence, binding, scope and authored range. With `position-index`,
`ExpressionPositionIndex` serves repeated editor requests from the same
immutable snapshot. One-shot requests can scan the existing tables.

Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

## License

MIT
