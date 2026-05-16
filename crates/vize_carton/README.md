# vize_carton

`vize_carton` is the shared foundation crate for the Vize workspace.

## Highlights

- Arena-backed allocation helpers: `Allocator`, `Bump`, `Box`, `Vec`, `CloneIn`
- Compact string and hash collection re-exports used throughout the workspace
- DOM tag tables and directive helpers
- Shared flags, profiler utilities, i18n helpers, and source-range utilities

## Common Exports

- `Allocator`, `Bump`, `Box`, `Vec`
- `CompactString`, `String`, `FxHashMap`, `FxHashSet`, `SmallVec`
- `is_html_tag`, `is_svg_tag`, `is_void_tag`, `is_builtin_directive`
- `PatchFlags`, `ShapeFlags`, `SlotFlags`

## Related Crates

Every other workspace crate depends on `vize_carton` either directly or indirectly.

## License

MIT
