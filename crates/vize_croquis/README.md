# vize_croquis

`vize_croquis` is the semantic analysis layer for Vue templates and SFCs.

## Highlights

- Scope tracking for template and script bindings
- Binding metadata used by compilers and type checking
- Reactivity and macro analysis
- Cross-file and virtual TypeScript support modules

## Key Entry Points

- `Analyzer`
- `AnalyzerOptions`
- `Croquis`
- `BindingMetadata`
- `ScopeChain`
- `SymbolTable`

## Related Crates

- `vize_armature` provides the parsed template tree
- `vize_atelier_dom`, `vize_atelier_vapor`, and `vize_atelier_ssr` consume binding metadata
- `vize_canon` and `vize_maestro` reuse the analysis layer for type-aware features

## License

MIT
