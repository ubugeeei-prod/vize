# vize_l0

`vize_l0` is the Davinci foundation level (L0): source, arena and span storage,
node and analysis ids, side tables, artifact keys, the dump trait and runtime,
diagnostics and witnesses, the pass and fact managers, and the level registry.

L0 owns the shared foundation implementation. The legacy `vize_carton` package
re-exports it from `crates/`, so Davinci has no reverse dependency on that tree.
Its allocator, storage, configuration and platform helpers keep their existing
behavior and std dependency. No-std isolation remains in #6834.

The remaining level restructure is tracked in
[#6833](https://github.com/ubugeeei-prod/vize/issues/6833) and
[#6834](https://github.com/ubugeeei-prod/vize/issues/6834). L0 has no skeleton bodies; the workspace ratchet prevents reintroducing them.

Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

## License

MIT
