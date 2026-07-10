# vize_croquis

`vize_croquis` is the semantic analysis layer for Vue templates and SFCs.

Relief records what source syntax was written and where. Croquis derives what
that syntax means and how it relates: symbol identity, scopes, bindings,
reactivity, usage, and analysis graphs. Cross-file aggregation is kept in the
separate `vize_croquis_cf` crate.

## Highlights

- Scope tracking for template and script bindings
- Binding metadata used by compilers and type checking
- Reactivity and macro analysis
- Call/effect graphs and virtual TypeScript support modules

## Key Entry Points

- `Drawer`
- `DrawerOptions`
- `Croquis`
- `BindingMetadata`
- `ScopeChain`
- `SymbolTable`

## Related Crates

- `vize_armature` provides the parsed template tree
- `vize_relief` owns source-faithful syntax nodes and locations
- `vize_croquis_cf` aggregates Croquis facts across files on demand
- `vize_atelier_dom`, `vize_atelier_vapor`, and `vize_atelier_ssr` consume binding metadata
- `vize_canon` and `vize_maestro` reuse the analysis layer for type-aware features

## License

MIT
