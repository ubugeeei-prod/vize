# vize_canon

`vize_canon` provides Vue-aware type checking and virtual TypeScript generation.

## Highlights

- Template-aware SFC diagnostics via `type_check_sfc`
- Virtual TypeScript generation and source maps
- Native batch checking backed by Corsa when the `native` feature is enabled
- Shared types for editor intelligence and type-aware services

## Key Entry Points

- `type_check_sfc`
- `TypeChecker`
- `TypeContext`
- `BatchTypeChecker` with the `native` feature
- `SourceMap`
- `VirtualTsGenerator` with the `native` feature

## Related Crates

- `vize_atelier_sfc` provides the SFC descriptor and binding metadata
- `vize_maestro` uses Canon for type-aware editor features
- `vize` exposes Canon through `vize check`

## License

MIT
