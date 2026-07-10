# vize_relief

`vize_relief` defines the shared AST, compiler errors, and compiler options used throughout the
Vize workspace.

Relief answers **what source syntax was written and where**. Nodes preserve
syntax shape and source locations; symbol identity, scope resolution,
dependencies, and control-flow facts belong to `vize_croquis`.

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
- `vize_croquis` derives meaning and relationships from the AST
- `vize_atelier_*`, `vize_patina`, and `vize_glyph` operate on the shared syntax model
- `vize_vitrine` serializes data derived from these types for JS consumers

## License

MIT
