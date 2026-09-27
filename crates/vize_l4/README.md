# vize_l4

`vize_l4` is Davinci's L4 emission level. Every output target writes text
directly into one append-only `write::Writer`: indentation, the used runtime
helpers, and span links that compile away when the writer does not record.
Preambles are joined after the body, so nothing is inserted mid-text.

`write::EmitDocument` and `write::SpanLink` are the finished document and its
generated-to-authored links; `write::source_map` serializes them as Source
Map v3. Legacy codegen re-exports these from `vize_atelier_core::codegen`.

Expression rewriting (`expr`), module assembly (`module`), the runtime helper
tables (`runtime`) and the DOM, SSR, Vapor and type-check targets (`targets`)
are unfinished skeletons tracked in
[#6840](https://github.com/ubugeeei-prod/vize/issues/6840); no product selects
them yet.

Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

## License

MIT
