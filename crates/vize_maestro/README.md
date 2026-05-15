# vize_maestro

`vize_maestro` is the Language Server Protocol implementation for Vize.

## Highlights

- StdIO and TCP language server entry points
- Virtual code generation for Vue SFCs
- IDE services for diagnostics, completion, hover, navigation, rename, and symbols

## Key Entry Points

- `serve`
- `serve_tcp`
- `MaestroServer`
- `IdeContext`
- `VirtualCodeGenerator`
- `VirtualDocuments`

## Related Crates

- `vize_patina` powers lint diagnostics
- `vize_canon` powers type-aware editor features
- `vize_glyph` powers formatting when enabled

## License

MIT
