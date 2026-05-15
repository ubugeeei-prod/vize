# vize_armature

`vize_armature` tokenizes and parses Vue template syntax into the `vize_relief` AST.

## Highlights

- Vue-aware HTML tokenizer with directive and interpolation support
- Parser entry points shared by the compiler, linter, formatter, and language server
- Parser options re-exported from `vize_relief`

## Key Entry Points

- `parse`
- `parse_with_options`
- `Parser`
- `tokenizer`

## Related Crates

- `vize_relief` defines the AST and parser options
- `vize_croquis` consumes the parsed tree for semantic analysis
- `vize_atelier_core` and `vize_atelier_*` build on the parser output

## License

MIT
