# vize_relief

`vize_relief` defines the shared AST, compiler errors, and compiler options used throughout the
Vize workspace.

## Highlights

- Vue template AST node definitions
- Parser, transform, and codegen options
- Shared compiler error types
- Arena-friendly data structures and serde support

## Main Modules

- `ast`
- `errors`
- `options`

## Related Crates

- `vize_armature` builds this AST
- `vize_atelier_*`, `vize_patina`, and `vize_glyph` operate on the shared syntax model
- `vize_vitrine` serializes data derived from these types for JS consumers

## License

MIT
