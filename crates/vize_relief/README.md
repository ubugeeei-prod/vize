# vize_relief

Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

`vize_relief` owns the source-faithful Vue-template syntax product plus its
parser-facing errors and options. It is not Vize's universal IR and is not used
by the JSX/TSX frontend.

Relief answers **what source syntax was written and where**. Nodes preserve
syntax shape and source locations. Symbol identity, scope resolution, and
reactivity belong to `vize_croquis`; control/data/effect flow belongs to
`vize_flow`; render intent belongs to `vize_rendu`.

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
- SFC providers can derive Croquis, Flow, or Rendu products from Relief
- syntax-oriented Patina and Glyph recipes may request Relief directly
- `vize_vitrine` serializes data derived from these types for JS consumers

## License

MIT
