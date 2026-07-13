# vize_maestro

`vize_maestro` is the Language Server Protocol implementation for Vize. It
owns one URI-keyed mutable Atlas `Compilation` for normal editor documents
instead of rebuilding a private analysis pipeline for each feature.

## Highlights

- StdIO and TCP language server entry points
- Stable Atlas source identities revised in place for open documents and
  file-backed Vue dependencies discovered by normal Vue editor requests
- Shared descriptor, Module, Relief, Croquis, Patina, Canon, virtual-document,
  and, when enabled, Glyph products requested independently from the persistent
  compilation
- Virtual code generation for Vue SFCs
- IDE services for diagnostics, completion, hover, navigation, rename, and symbols

For the normal type-aware `.vue` path, Maestro queries Canon's
`CanonVueDocumentProduct` from that same compilation. It recursively joins
open overlays and on-disk Vue dependencies to the persistent source set, then
passes the prebuilt host document and prebuilt dependency overlays to Corsa.
Corsa does not create a private Atlas compilation or reparse those SFCs.

## Key Entry Points

- `serve`
- `serve_tcp`
- `MaestroServer`
- `IdeContext`
- `VirtualCodeGenerator`
- `VirtualDocuments`

## Related Crates

- `vize_patina` powers lint diagnostics
- `vize_canon` owns the typed virtual-document product synchronized to Corsa
- `vize_glyph` provides the formatter product queried from the same
  compilation when enabled
- `vize_atlas` owns source identity, planning, caching, and invalidation

## License

MIT
