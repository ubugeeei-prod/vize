# vize_canon

Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

`vize_canon` provides Vue-aware type checking and virtual TypeScript
generation. Its editor-facing Atlas product consumes existing source-shaped
products rather than owning a parser or universal compiler representation.

## Highlights

- Template-aware SFC diagnostics via `type_check_sfc`
- Virtual TypeScript generation and source maps
- Native batch checking backed by Corsa when the `native` feature is enabled
- Parser-free `CanonVueDocumentProduct`: descriptor and Croquis are always
  requested, Relief only when a template exists, and SFC script syntax plus
  Module only when a script exists
- Shared types for editor intelligence and type-aware services

In the normal native `.vue` editor path, Maestro queries that product for the
host and transitive open or on-disk Vue dependencies from its one persistent
compilation. Canon then gives Corsa the prebuilt host document and prebuilt Vue
dependency overlays. Corsa does not create a private Atlas compilation or
reparse those SFCs. Specialized Art and Musea paths are separate from this
normal Vue production path.

## Key Entry Points

- `batch::CanonVueDocumentProduct` with the `native` feature
- `batch::register_canon_vue_document_provider` with the `native` feature
- `type_check_sfc`
- `TypeChecker`
- `TypeContext`
- `BatchTypeChecker` with the `native` feature
- `SourceMap`
- `VirtualTsGenerator` with the `native` feature

## Related Crates

- `vize_atelier_sfc` provides descriptor and authored-script products
- `vize_croquis` provides frontend-neutral semantic meaning
- `vize_maestro` queries Canon from its persistent URI-keyed compilation
- `vize` exposes Canon through `vize check`

## License

MIT
