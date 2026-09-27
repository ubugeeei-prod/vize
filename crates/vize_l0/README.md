# vize_l0

`vize_l0` is the Davinci foundation level (L0): source, arena and span storage,
node and analysis ids, side tables, artifact keys, the dump trait and runtime,
diagnostics and witnesses, the pass and fact managers, and the level registry.

The crate is a compiling skeleton ([#6833](https://github.com/ubugeeei-prod/vize/issues/6833),
[#6834](https://github.com/ubugeeei-prod/vize/issues/6834)). Unimplemented bodies are
`todo!()` inside modules that carry `#![expect(clippy::todo, reason = "skeleton: #NNNN")]`;
`tools/commands/ci/check-skeleton-todos.rs` ratchets their count down as code moves in
from `vize_davinci` and `vize_carton`. The `vize_l0` workspace dependency resolves to this
package, which re-exports `vize_carton`'s public API until #6834 moves that storage here.

Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

## License

MIT
